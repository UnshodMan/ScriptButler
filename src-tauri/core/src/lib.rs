pub mod exec;
pub mod language;
pub mod models;
pub mod ops;
pub mod search;
pub mod store;

/// 当前时间（Unix 毫秒）
pub fn unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
