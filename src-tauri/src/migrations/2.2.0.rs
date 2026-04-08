use serde_json::Value;

type PendingStep = fn(Value) -> Result<Value, String>;

const PENDING_STEPS: &[PendingStep] = &[
    // Add pending migration steps here, in order.
    migrate_instrument_playback_routes,
];

fn max_ids_in_array(value: &Value) -> u64 {
    value
        .as_array()
        .map(|values| values.iter().filter_map(Value::as_u64).max().unwrap_or(0))
        .unwrap_or(0)
}

fn max_runtime_id(value: &Value) -> u64 {
    match value {
        Value::Array(values) => values.iter().map(max_runtime_id).max().unwrap_or(0),
        Value::Object(values) => {
            let mut max_id = 0;

            for (key, current) in values {
                if key == "by_id" {
                    max_id = max_id.max(
                        current
                            .as_object()
                            .map(|entries| {
                                entries
                                    .keys()
                                    .filter_map(|id| id.parse::<u64>().ok())
                                    .max()
                                    .unwrap_or(0)
                            })
                            .unwrap_or(0),
                    );
                }

                if key == "id" || key == "next_id" || key.ends_with("_id") {
                    max_id = max_id.max(current.as_u64().unwrap_or(0));
                }

                if key.ends_with("_ids") || key == "order" {
                    max_id = max_id.max(max_ids_in_array(current));
                }

                max_id = max_id.max(max_runtime_id(current));
            }

            max_id
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => 0,
    }
}

fn next_slot_id(next_id: &mut u64) -> u64 {
    let slot_id = *next_id;
    *next_id += 1;
    slot_id
}

fn playback_route_from_legacy(
    plugin_id: String,
    target: Option<&serde_json::Map<String, Value>>,
    slot_id: u64,
) -> Value {
    let (target_mode, target_index, address_mode, address_index) =
        match target.and_then(|target| target.get("mode").and_then(Value::as_u64)) {
            Some(1) => (
                2_u64,
                target
                    .and_then(|current| current.get("index"))
                    .and_then(Value::as_u64),
                2_u64,
                target
                    .and_then(|current| current.get("index"))
                    .and_then(Value::as_u64),
            ),
            Some(_) => (
                1_u64,
                target
                    .and_then(|current| current.get("index"))
                    .and_then(Value::as_u64),
                1_u64,
                target
                    .and_then(|current| current.get("index"))
                    .and_then(Value::as_u64),
            ),
            None => (0_u64, None, 0_u64, None),
        };

    let _ = plugin_id;

    serde_json::json!({
        "slot_id": slot_id,
        "target": {
            "mode": target_mode,
            "index": target_index,
        },
        "address": {
            "mode": address_mode,
            "index": address_index,
        }
    })
}

fn migrate_instrument_playback_routes(mut data: Value) -> Result<Value, String> {
    let mut next_id = data
        .get("next_id")
        .and_then(Value::as_u64)
        .unwrap_or_else(|| max_runtime_id(&data).saturating_add(1));
    let mut shared_slot_ids: std::collections::BTreeMap<u64, u64> =
        std::collections::BTreeMap::new();

    let existing_playback = data
        .get("score")
        .and_then(|score| score.get("playback"))
        .cloned()
        .unwrap_or_else(|| {
            serde_json::json!({
                "rack": {
                    "order": [],
                    "by_id": {}
                }
            })
        });

    let mut slot_order = existing_playback
        .get("rack")
        .and_then(|rack| rack.get("order"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut slot_by_id = existing_playback
        .get("rack")
        .and_then(|rack| rack.get("by_id"))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

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

        if instrument_object.contains_key("playback_route") {
            instrument_object.remove("assignment_intent");
            instrument_object.remove("assigned_plugin_id");
            continue;
        }

        let assignment_intent = instrument_object.remove("assignment_intent");
        let legacy_assigned_plugin_id = instrument_object.remove("assigned_plugin_id");

        let (plugin_id, target, shared_instance_id, mode) =
            match assignment_intent.as_ref().and_then(Value::as_object) {
                Some(intent) => (
                    intent
                        .get("plugin_id")
                        .and_then(Value::as_str)
                        .map(String::from)
                        .or_else(|| {
                            legacy_assigned_plugin_id
                                .as_ref()
                                .and_then(Value::as_str)
                                .map(String::from)
                        }),
                    intent.get("target").and_then(Value::as_object),
                    intent.get("shared_instance_id").and_then(Value::as_u64),
                    intent.get("mode").and_then(Value::as_u64).unwrap_or(0),
                ),
                None => (
                    legacy_assigned_plugin_id
                        .as_ref()
                        .and_then(Value::as_str)
                        .map(String::from),
                    None,
                    None,
                    0,
                ),
            };

        if mode == 0 || plugin_id.is_none() {
            instrument_object.insert(String::from("playback_route"), Value::Null);
            continue;
        }

        let plugin_id = plugin_id.unwrap();
        let slot_id = match shared_instance_id {
            Some(shared_instance_id) => *shared_slot_ids
                .entry(shared_instance_id)
                .or_insert_with(|| next_slot_id(&mut next_id)),
            None => next_slot_id(&mut next_id),
        };

        let slot_key = slot_id.to_string();
        if !slot_by_id.contains_key(&slot_key) {
            slot_order.push(Value::Number(slot_id.into()));
            slot_by_id.insert(
                slot_key,
                serde_json::json!({
                    "id": slot_id,
                    "target": {
                        "plugin_id": plugin_id,
                    }
                }),
            );
        }

        instrument_object.insert(
            String::from("playback_route"),
            playback_route_from_legacy(plugin_id, target, slot_id),
        );
    }

    if let Some(score_object) = data.get_mut("score").and_then(Value::as_object_mut) {
        score_object.insert(
            String::from("playback"),
            serde_json::json!({
                "rack": {
                    "order": slot_order,
                    "by_id": slot_by_id,
                }
            }),
        );
    }

    if let Some(root_object) = data.as_object_mut() {
        root_object.insert(String::from("next_id"), Value::Number(next_id.into()));
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
    fn migrates_assignment_intent_to_playback_routes() {
        let data = json!({
            "next_id": 10,
            "score": {
                "instruments": {
                    "by_id": {
                        "1": {
                            "assignment_intent": {
                                "mode": 1,
                                "plugin_id": "plugin-a",
                                "target": null,
                                "shared_instance_id": null
                            }
                        },
                        "2": {
                            "assignment_intent": {
                                "mode": 0,
                                "plugin_id": null,
                                "target": null,
                                "shared_instance_id": null
                            }
                        }
                    }
                }
            }
        });

        let migrated = migrate(data).unwrap();

        assert_eq!(
            migrated["score"]["instruments"]["by_id"]["1"]["playback_route"]["slot_id"],
            Value::Number(10.into())
        );
        assert_eq!(
            migrated["score"]["playback"]["rack"]["by_id"]["10"]["target"]["plugin_id"],
            Value::String(String::from("plugin-a"))
        );
        assert_eq!(
            migrated["score"]["instruments"]["by_id"]["2"]["playback_route"],
            Value::Null
        );
        assert_eq!(migrated["next_id"], Value::Number(11.into()));
        assert_eq!(
            migrated["score"]["instruments"]["by_id"]["1"].get("assigned_plugin_id"),
            None
        );
        assert_eq!(
            migrated["score"]["instruments"]["by_id"]["1"].get("assignment_intent"),
            None
        );
    }
}
