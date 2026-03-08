use serde_json::Value;

pub fn migrate(mut data: Value) -> Result<Value, String> {
    let Some(tracks) = data
        .get_mut("score")
        .and_then(|score| score.get_mut("tracks"))
        .and_then(|tracks| tracks.get_mut("by_id"))
        .and_then(Value::as_object_mut)
    else {
        return Ok(data);
    };

    for track in tracks.values_mut() {
        let Some(entries) = track
            .get_mut("entries")
            .and_then(|entries| entries.get_mut("by_id"))
            .and_then(Value::as_object_mut)
        else {
            continue;
        };

        for entry in entries.values_mut() {
            let is_tempo = entry
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind == "Tempo");

            if !is_tempo {
                continue;
            }

            match entry.get_mut("text") {
                Some(text) if text.is_null() => *text = Value::String(String::new()),
                None => {
                    if let Some(object) = entry.as_object_mut() {
                        object.insert(String::from("text"), Value::String(String::new()));
                    }
                }
                _ => {}
            }
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::migrate;
    use serde_json::json;

    #[test]
    fn normalizes_null_tempo_text_to_empty_string() {
        let migrated = migrate(json!({
            "score": {
                "tracks": {
                    "by_id": {
                        "1": {
                            "entries": {
                                "by_id": {
                                    "10": {
                                        "type": "Tempo",
                                        "text": null
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }))
        .unwrap();

        assert_eq!(
            migrated["score"]["tracks"]["by_id"]["1"]["entries"]["by_id"]["10"]["text"],
            ""
        );
    }
}
