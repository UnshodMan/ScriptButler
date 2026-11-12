use std::io::{BufRead, BufReader, Read};
use std::process::Stdio;
use std::thread;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

use scriptbutler_core::exec::{build_run_command, shell_command};

use crate::models::{ApiError, RunEvent, RunPayload};
use crate::state::AppState;

const EVENT: &str = "run://event";

/// 启动脚本：解析方案与执行目录、拼接命令（含临时参数）并 spawn 子进程
pub fn start_run(
    app: AppHandle,
    state: &AppState,
    path: &str,
    scheme_id: &str,
    args: Option<&str>,
) -> Result<String, ApiError> {
    let cmdline = {
        let entries = state.lock_entries();
        let entry = entries
            .iter()
            .find(|e| e.path == path)
            .ok_or(ApiError::NotFound)?;
        let scheme = entry
            .schemes
            .iter()
            .find(|s| s.id == scheme_id)
            .ok_or(ApiError::NotFound)?;
        build_run_command(entry, scheme, args)
    };

    let mut cmd = shell_command(&cmdline);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| ApiError::SpawnFailed {
        detail: e.to_string(),
    })?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let run_id = scriptbutler_core::ops::new_id();

    let out_thread = stdout.map(|s| spawn_reader(app.clone(), run_id.clone(), s, true));
    let err_thread = stderr.map(|s| spawn_reader(app.clone(), run_id.clone(), s, false));

    state
        .children
        .lock()
        .unwrap()
        .insert(run_id.clone(), child);

    let app_for_wait = app.clone();
    let rid = run_id.clone();
    thread::spawn(move || {
        let mut out_thread = out_thread;
        let mut err_thread = err_thread;
        loop {
            thread::sleep(Duration::from_millis(100));
            let state = app_for_wait.state::<AppState>();
            let mut children = state.children.lock().unwrap();
            let status = match children.get_mut(&rid) {
                Some(child) => child.try_wait(),
                None => return,
            };
            match status {
                Ok(None) => {}
                Ok(Some(status)) => {
                    children.remove(&rid);
                    drop(children);
                    // 给读取线程短暂时间冲刷剩余输出
                    wait_readers(&mut out_thread, &mut err_thread);
                    let _ = app_for_wait.emit(
                        EVENT,
                        RunPayload {
                            run_id: rid.clone(),
                            event: RunEvent::Exit {
                                code: status.code(),
                            },
                        },
                    );
                    return;
                }
                Err(e) => {
                    children.remove(&rid);
                    drop(children);
                    let _ = app_for_wait.emit(
                        EVENT,
                        RunPayload {
                            run_id: rid.clone(),
                            event: RunEvent::Error {
                                message: e.to_string(),
                            },
                        },
                    );
                    return;
                }
            }
        }
    });

    Ok(run_id)
}

pub fn stop_run(state: &AppState, run_id: &str) {
    let mut children = state.children.lock().unwrap();
    if let Some(child) = children.get_mut(run_id) {
        let _ = child.kill();
    }
}

fn spawn_reader(
    app: AppHandle,
    run_id: String,
    stream: impl Read + Send + 'static,
    is_stdout: bool,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut reader = BufReader::new(stream);
        let mut buf = Vec::new();
        loop {
            buf.clear();
            match reader.read_until(b'\n', &mut buf) {
                Ok(0) => return,
                Ok(_) => {
                    if buf.last() == Some(&b'\n') {
                        buf.pop();
                    }
                    if buf.last() == Some(&b'\r') {
                        buf.pop();
                    }
                    let line = String::from_utf8_lossy(&buf).to_string();
                    let event = if is_stdout {
                        RunEvent::Stdout { line }
                    } else {
                        RunEvent::Stderr { line }
                    };
                    let _ = app.emit(
                        EVENT,
                        RunPayload {
                            run_id: run_id.clone(),
                            event,
                        },
                    );
                }
                Err(_) => return,
            }
        }
    })
}

/// 等待读取线程结束，最多 500ms，避免后台孙进程长期占用管道导致等待线程悬挂
fn wait_readers(
    out: &mut Option<thread::JoinHandle<()>>,
    err: &mut Option<thread::JoinHandle<()>>,
) {
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(500) {
        let out_done = out.as_ref().map(|h| h.is_finished()).unwrap_or(true);
        let err_done = err.as_ref().map(|h| h.is_finished()).unwrap_or(true);
        if out_done && err_done {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
}
