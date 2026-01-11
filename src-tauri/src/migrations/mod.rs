mod v1_19_0;

use lazy_static::lazy_static;

type Migration = (
    &'static str,
    fn(serde_json::Value) -> Result<serde_json::Value, String>,
);

lazy_static! {
    pub static ref MIGRATIONS: Vec<Migration> = vec![("1.19.0", v1_19_0::migrate),];
}
