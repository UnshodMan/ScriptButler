use std::path::Path;

use serde_json::json;

use crate::output::{self, CliError};

#[derive(Clone, Copy)]
pub enum EnvAction {
    Add,
    Remove,
    Status,
}

impl EnvAction {
    fn as_str(self) -> &'static str {
        match self {
            EnvAction::Add => "add",
            EnvAction::Remove => "remove",
            EnvAction::Status => "status",
        }
    }
}

pub fn run(action: EnvAction, system: bool, json: bool) -> Result<(), CliError> {
    #[cfg(windows)]
    {
        windows::run(action, system, json)
    }
    #[cfg(not(windows))]
    {
        unix::run(action, system, json)
    }
}

pub(crate) fn report(
    action: EnvAction,
    executable: &Path,
    directory: &Path,
    on_path: bool,
    message: String,
    json: bool,
) {
    if json {
        output::print_json(json!({
            "ok": true,
            "data": {
                "action": action.as_str(),
                "executable": executable.to_string_lossy(),
                "directory": directory.to_string_lossy(),
                "onPath": on_path,
                "message": message
            }
        }));
    } else {
        println!("{message}");
    }
}

#[cfg(not(windows))]
mod unix {
    use std::path::{Path, PathBuf};

    use super::{report, CliError, EnvAction};

    pub fn run(action: EnvAction, system: bool, json: bool) -> Result<(), CliError> {
        if system {
            return Err(CliError::Io(
                "--system is only supported on Windows".to_string(),
            ));
        }

        let exe = std::env::current_exe().map_err(|e| CliError::Io(e.to_string()))?;
        let dir = exe
            .parent()
            .ok_or_else(|| CliError::Io("cannot resolve executable directory".to_string()))?
            .to_path_buf();

        if std::env::var_os("APPIMAGE").is_some() || exe.to_string_lossy().contains("/.mount_") {
            return Err(CliError::Io(
                "sbctl is running from an AppImage; use the deb/rpm package or add the binary to PATH manually"
                    .to_string(),
            ));
        }

        let home = std::env::var_os("HOME")
            .ok_or_else(|| CliError::Io("HOME is not set".to_string()))?;
        let link = PathBuf::from(home).join(".local/bin/sbctl");
        let bin_dir = link
            .parent()
            .ok_or_else(|| CliError::Io("cannot resolve ~/.local/bin".to_string()))?
            .to_path_buf();

        let dir_on_path = path_contains(&dir);
        let link_ok = link_points_to(&link, &exe);

        match action {
            EnvAction::Status => {
                let on_path = dir_on_path || (path_contains(&bin_dir) && link_ok);
                let message = if dir_on_path {
                    format!("{} is already on PATH", dir.display())
                } else if on_path {
                    format!("{} is on PATH via {}", exe.display(), link.display())
                } else {
                    format!(
                        "{} is not on PATH; run 'sbctl env add' or add it manually",
                        dir.display()
                    )
                };
                report(action, &exe, &dir, on_path, message, json);
            }
            EnvAction::Add => {
                if dir_on_path {
                    report(
                        action,
                        &exe,
                        &dir,
                        true,
                        format!("{} is already on PATH", dir.display()),
                        json,
                    );
                    return Ok(());
                }

                if let Ok(meta) = std::fs::symlink_metadata(&link) {
                    if !meta.file_type().is_symlink() {
                        return Err(CliError::Io(format!(
                            "{} already exists and is not a symlink",
                            link.display()
                        )));
                    }
                    if !link_ok {
                        return Err(CliError::Io(format!(
                            "{} already exists and points elsewhere; remove it first",
                            link.display()
                        )));
                    }
                }

                std::fs::create_dir_all(&bin_dir).map_err(|e| CliError::Io(e.to_string()))?;
                if std::fs::symlink_metadata(&link).is_err() {
                    std::os::unix::fs::symlink(&exe, &link)
                        .map_err(|e| CliError::Io(e.to_string()))?;
                }

                let on_path = path_contains(&bin_dir);
                let message = if on_path {
                    format!("Linked {} -> {}", link.display(), exe.display())
                } else {
                    format!(
                        "Linked {} -> {}\nAdd ~/.local/bin to PATH, e.g. export PATH=\"$HOME/.local/bin:$PATH\"",
                        link.display(),
                        exe.display()
                    )
                };
                report(action, &exe, &dir, on_path, message, json);
            }
            EnvAction::Remove => {
                let mut removed = false;
                if let Ok(meta) = std::fs::symlink_metadata(&link) {
                    if meta.file_type().is_symlink() && link_points_to(&link, &exe) {
                        std::fs::remove_file(&link).map_err(|e| CliError::Io(e.to_string()))?;
                        removed = true;
                    }
                }
                let message = if removed {
                    format!("Removed {}", link.display())
                } else {
                    format!("No sbctl symlink to remove at {}", link.display())
                };
                report(action, &exe, &dir, path_contains(&dir), message, json);
            }
        }
        Ok(())
    }

    fn link_points_to(link: &Path, exe: &Path) -> bool {
        std::fs::read_link(link)
            .map(|target| target == exe || same_file(&target, exe))
            .unwrap_or(false)
    }

    fn same_file(a: &Path, b: &Path) -> bool {
        match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }

    fn path_contains(dir: &Path) -> bool {
        let Some(path) = std::env::var_os("PATH") else {
            return false;
        };
        std::env::split_paths(&path).any(|entry| entry == dir || same_file(&entry, dir))
    }
}

#[cfg(windows)]
mod windows {
    use std::path::Path;
    use std::process::Command;

    use super::{report, CliError, EnvAction};

    pub fn run(action: EnvAction, system: bool, json: bool) -> Result<(), CliError> {
        let exe = std::env::current_exe().map_err(|e| CliError::Io(e.to_string()))?;
        let dir = exe
            .parent()
            .ok_or_else(|| CliError::Io("cannot resolve executable directory".to_string()))?
            .to_path_buf();
        let scope = if system { "Machine" } else { "User" };

        match action {
            EnvAction::Status => {
                let on_path = path_contains(&dir, scope)?;
                let message = if on_path {
                    format!("{} is on {} PATH", dir.display(), scope)
                } else {
                    format!(
                        "{} is not on {} PATH; run 'sbctl env add'",
                        dir.display(),
                        scope
                    )
                };
                report(action, &exe, &dir, on_path, message, json);
            }
            EnvAction::Add => {
                set_path(&dir, scope, true)?;
                let on_path = path_contains(&dir, scope)?;
                report(
                    action,
                    &exe,
                    &dir,
                    on_path,
                    format!(
                        "Added {} to {} PATH (restart terminals to apply)",
                        dir.display(),
                        scope
                    ),
                    json,
                );
            }
            EnvAction::Remove => {
                set_path(&dir, scope, false)?;
                report(
                    action,
                    &exe,
                    &dir,
                    false,
                    format!("Removed {} from {} PATH", dir.display(), scope),
                    json,
                );
            }
        }
        Ok(())
    }

    fn path_contains(dir: &Path, scope: &str) -> Result<bool, CliError> {
        let script = format!(
            "$p=[Environment]::GetEnvironmentVariable('Path','{scope}'); if($null -eq $p){{$p=''}}; if(@($p.Split(';',[System.StringSplitOptions]::RemoveEmptyEntries)) -contains '{}'){{'yes'}}else{{'no'}}",
            escape(dir)
        );
        let out = run_powershell(&script)?;
        Ok(out.trim() == "yes")
    }

    fn set_path(dir: &Path, scope: &str, add: bool) -> Result<(), CliError> {
        let escaped = escape(dir);
        let script = if add {
            format!(
                "$d='{escaped}'; $s='{scope}'; $p=[Environment]::GetEnvironmentVariable('Path',$s); if($null -eq $p){{$p=''}}; $parts=@($p.Split(';',[System.StringSplitOptions]::RemoveEmptyEntries)); if($parts -notcontains $d){{[Environment]::SetEnvironmentVariable('Path',(($parts + $d) -join ';'),$s)}}"
            )
        } else {
            format!(
                "$d='{escaped}'; $s='{scope}'; $p=[Environment]::GetEnvironmentVariable('Path',$s); if($null -ne $p){{$parts=@($p.Split(';',[System.StringSplitOptions]::RemoveEmptyEntries) | Where-Object {{$_ -ne $d}}); [Environment]::SetEnvironmentVariable('Path',($parts -join ';'),$s)}}"
            )
        };
        run_powershell(&script).map(|_| ())
    }

    fn run_powershell(script: &str) -> Result<String, CliError> {
        let out = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .output()
            .map_err(|e| CliError::Io(format!("failed to run PowerShell: {e}")))?;
        if !out.status.success() {
            return Err(CliError::Io(format!(
                "PowerShell failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    fn escape(dir: &Path) -> String {
        dir.to_string_lossy().replace('\'', "''")
    }
}
