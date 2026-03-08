#[path = "1.25.0.rs"]
mod v1_25_0;

use lazy_static::lazy_static;

type Migration = (
    &'static str,
    fn(serde_json::Value) -> Result<serde_json::Value, String>,
);

lazy_static! {
    pub static ref MIGRATIONS: Vec<Migration> = vec![("1.25.0", v1_25_0::migrate)];
}
