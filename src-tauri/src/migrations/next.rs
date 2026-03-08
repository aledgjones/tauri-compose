use serde_json::Value;

type PendingStep = fn(Value) -> Result<Value, String>;

const PENDING_STEPS: &[PendingStep] = &[
    // Add pending migration steps here, in order.
];

pub fn migrate(mut data: Value) -> Result<Value, String> {
    for step in PENDING_STEPS {
        data = step(data)?;
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::migrate;
    use serde_json::json;

    #[test]
    fn scaffold_migration_is_a_noop() {
        let data = json!({
            "score": {
                "meta": {
                    "app_version": env!("CARGO_PKG_VERSION")
                }
            }
        });

        assert_eq!(migrate(data.clone()).unwrap(), data);
    }
}
