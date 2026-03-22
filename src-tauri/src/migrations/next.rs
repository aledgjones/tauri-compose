use serde_json::Value;

type PendingStep = fn(Value) -> Result<Value, String>;

const PENDING_STEPS: &[PendingStep] = &[
    // Add pending migration steps here, in order.
    migrate_instrument_assignment_intent,
];

fn migrate_instrument_assignment_intent(mut data: Value) -> Result<Value, String> {
    let Some(instruments_by_id) = data
        .get_mut("score")
        .and_then(|score| score.get_mut("instruments"))
        .and_then(|instruments| instruments.get_mut("by_id"))
        .and_then(Value::as_object_mut)
    else {
        return Ok(data);
    };

    for instrument in instruments_by_id.values_mut() {
        let Some(instrument_object) = instrument.as_object_mut() else {
            continue;
        };

        if instrument_object.contains_key("assignment_intent") {
            instrument_object.remove("assigned_plugin_id");
            continue;
        }

        let plugin_id = instrument_object.remove("assigned_plugin_id").unwrap_or(Value::Null);
        let assignment_intent = if plugin_id.is_null() {
            serde_json::json!({
                "mode": 0,
                "plugin_id": Value::Null,
                "target": Value::Null,
                "shared_instance_id": Value::Null,
            })
        } else {
            serde_json::json!({
                "mode": 1,
                "plugin_id": plugin_id,
                "target": Value::Null,
                "shared_instance_id": Value::Null,
            })
        };

        instrument_object.insert(String::from("assignment_intent"), assignment_intent);
    }

    Ok(data)
}

pub fn migrate(mut data: Value) -> Result<Value, String> {
    for step in PENDING_STEPS {
        data = step(data)?;
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::migrate;
    use serde_json::{json, Value};

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

    #[test]
    fn migrates_legacy_assigned_plugin_id_to_assignment_intent() {
        let data = json!({
            "score": {
                "instruments": {
                    "by_id": {
                        "1": {
                            "assigned_plugin_id": "plugin-a"
                        },
                        "2": {
                            "assigned_plugin_id": null
                        }
                    }
                }
            }
        });

        let migrated = migrate(data).unwrap();

        assert_eq!(
            migrated["score"]["instruments"]["by_id"]["1"]["assignment_intent"]["mode"],
            Value::Number(1.into())
        );
        assert_eq!(
            migrated["score"]["instruments"]["by_id"]["1"]["assignment_intent"]["plugin_id"],
            Value::String(String::from("plugin-a"))
        );
        assert_eq!(
            migrated["score"]["instruments"]["by_id"]["2"]["assignment_intent"]["mode"],
            Value::Number(0.into())
        );
        assert_eq!(
            migrated["score"]["instruments"]["by_id"]["1"].get("assigned_plugin_id"),
            None
        );
    }
}
