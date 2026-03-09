use serde_json::Value;

type PendingStep = fn(Value) -> Result<Value, String>;

const PENDING_STEPS: &[PendingStep] = &[
    // Add pending migration steps here, in order.
];

fn migrate_directive_hidden_flags(mut data: Value) -> Result<Value, String> {
    let Some(entries_by_track) = data
        .get_mut("score")
        .and_then(|score| score.get_mut("tracks"))
        .and_then(|tracks| tracks.get_mut("by_id"))
        .and_then(Value::as_object_mut)
    else {
        return Ok(data);
    };

    for track in entries_by_track.values_mut() {
        let Some(entries_by_id) = track
            .get_mut("entries")
            .and_then(|entries| entries.get_mut("by_id"))
            .and_then(Value::as_object_mut)
        else {
            continue;
        };

        for entry in entries_by_id.values_mut() {
            let Some(entry_object) = entry.as_object_mut() else {
                continue;
            };

            match entry_object.get("type").and_then(Value::as_str) {
                Some("Clef") | Some("KeySignature") | Some("TimeSignature") => {
                    if !entry_object.contains_key("hidden") {
                        entry_object.insert(String::from("hidden"), Value::Bool(false));
                    }
                }
                _ => {}
            }

            if entry_object.get("type").and_then(Value::as_str) == Some("TimeSignature")
                && entry_object.get("draw_type").and_then(Value::as_u64) == Some(0)
            {
                entry_object.insert(String::from("hidden"), Value::Bool(true));
                entry_object.insert(
                    String::from("draw_type"),
                    Value::Number(serde_json::Number::from(1)),
                );
            }
        }
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
    fn migrates_hidden_time_signature_draw_type_to_hidden_flag() {
        let data = json!({
            "score": {
                "tracks": {
                    "by_id": {
                        "1": {
                            "entries": {
                                "by_id": {
                                    "2": {
                                        "type": "TimeSignature",
                                        "draw_type": 0
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        let migrated = migrate(data).unwrap();
        let entry = &migrated["score"]["tracks"]["by_id"]["1"]["entries"]["by_id"]["2"];

        assert_eq!(entry["hidden"], Value::Bool(true));
        assert_eq!(entry["draw_type"], Value::Number(1.into()));
    }
}
