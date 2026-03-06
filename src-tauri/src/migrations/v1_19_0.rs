pub fn migrate(data: serde_json::Value) -> Result<serde_json::Value, String> {
    let mut data = data;

    if let Some(score) = data.get_mut("score") {
        if let Some(tracks) = score.get_mut("tracks") {
            if let Some(tracks_by_id) = tracks.get_mut("by_id") {
                for (_, track) in tracks_by_id.as_object_mut().unwrap().iter_mut() {
                    if let Some(entries) = track.get_mut("entries") {
                        if let Some(entries_by_id) = entries.get_mut("by_id") {
                            for (_, entry) in entries_by_id.as_object_mut().unwrap().iter_mut() {
                                // move entries
                                if let Some(tick_value) = entry.get_mut("tick") {
                                    if let Some(tick) = tick_value.as_u64() {
                                        let new_tick = tick / 8;
                                        *tick_value = serde_json::Value::Number(
                                            serde_json::Number::from(new_tick),
                                        );
                                    }
                                }

                                // compress duration
                                if let Some(duration_value) = entry.get_mut("duration") {
                                    if let Some(duration) = duration_value.as_u64() {
                                        let new_duration = duration / 8;
                                        *duration_value = serde_json::Value::Number(
                                            serde_json::Number::from(new_duration),
                                        );
                                    }
                                }
                            }
                        }

                        // remap the by_tick entries {[tick]: id[]}
                        if let Some(entries_by_tick) = entries.get_mut("by_tick") {
                            let mut mapped = serde_json::Map::new();
                            for (tick, entry) in entries_by_tick.as_object_mut().unwrap().iter_mut()
                            {
                                let new_tick = tick.parse::<u64>().unwrap() / 8;
                                mapped.insert(new_tick.to_string(), entry.clone());
                            }
                            *entries_by_tick = serde_json::Value::Object(mapped);
                        }
                    }
                }
            }
        }

        // reduce the length of each flow
        if let Some(flows) = score.get_mut("flows") {
            if let Some(flows_by_id) = flows.get_mut("by_id") {
                for (_, flow) in flows_by_id.as_object_mut().unwrap().iter_mut() {
                    if let Some(length_value) = flow.get_mut("length") {
                        if let Some(length) = length_value.as_u64() {
                            let new_length = length / 8;
                            *length_value =
                                serde_json::Value::Number(serde_json::Number::from(new_length));
                        }
                    }
                }
            }
        }

        // reset the renderes pan and zoom
        if let Some(ui) = score.get_mut("ui") {
            if let Some(renderer) = ui.get_mut("renderer") {
                if let Some(zoom_value) = renderer.get_mut("zoom") {
                    *zoom_value =
                        serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap());
                }
                if let Some(pan_x_value) = renderer.get_mut("pan_x") {
                    *pan_x_value =
                        serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap());
                }
                if let Some(pan_y_value) = renderer.get_mut("pan_y") {
                    *pan_y_value =
                        serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap());
                }
            }
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn migrate_passthrough_when_no_score_key() {
        let data = json!({"other": 1});
        let result = migrate(data).unwrap();
        assert_eq!(result["other"], 1);
    }

    #[test]
    fn migrate_reduces_tick_by_factor_8() {
        let data = json!({
            "score": {
                "tracks": {
                    "by_id": {
                        "t1": {
                            "entries": {
                                "by_id": {
                                    "e1": {"tick": 128u64, "duration": 64u64}
                                },
                                "by_tick": {
                                    "128": ["e1"]
                                }
                            }
                        }
                    }
                }
            }
        });
        let result = migrate(data).unwrap();
        let tick = &result["score"]["tracks"]["by_id"]["t1"]["entries"]["by_id"]["e1"]["tick"];
        assert_eq!(tick, 16);
    }

    #[test]
    fn migrate_reduces_duration_by_factor_8() {
        let data = json!({
            "score": {
                "tracks": {
                    "by_id": {
                        "t1": {
                            "entries": {
                                "by_id": {
                                    "e1": {"tick": 8u64, "duration": 64u64}
                                },
                                "by_tick": {
                                    "8": ["e1"]
                                }
                            }
                        }
                    }
                }
            }
        });
        let result = migrate(data).unwrap();
        let dur = &result["score"]["tracks"]["by_id"]["t1"]["entries"]["by_id"]["e1"]["duration"];
        assert_eq!(dur, 8);
    }

    #[test]
    fn migrate_remaps_by_tick_keys() {
        let data = json!({
            "score": {
                "tracks": {
                    "by_id": {
                        "t1": {
                            "entries": {
                                "by_id": {
                                    "e1": {"tick": 16u64, "duration": 8u64}
                                },
                                "by_tick": {
                                    "16": ["e1"]
                                }
                            }
                        }
                    }
                }
            }
        });
        let result = migrate(data).unwrap();
        let by_tick = &result["score"]["tracks"]["by_id"]["t1"]["entries"]["by_tick"];
        // old key "16" / 8 = "2"
        assert!(by_tick.get("2").is_some());
        assert!(by_tick.get("16").is_none());
    }

    #[test]
    fn migrate_reduces_flow_length() {
        let data = json!({
            "score": {
                "flows": {
                    "by_id": {
                        "f1": {"length": 384u64}
                    }
                }
            }
        });
        let result = migrate(data).unwrap();
        let len = &result["score"]["flows"]["by_id"]["f1"]["length"];
        assert_eq!(len, 48);
    }

    #[test]
    fn migrate_resets_renderer_pan_and_zoom() {
        let data = json!({
            "score": {
                "ui": {
                    "renderer": {
                        "zoom": 150.0,
                        "pan_x": 200.0,
                        "pan_y": 300.0
                    }
                }
            }
        });
        let result = migrate(data).unwrap();
        let zoom = &result["score"]["ui"]["renderer"]["zoom"];
        let pan_x = &result["score"]["ui"]["renderer"]["pan_x"];
        let pan_y = &result["score"]["ui"]["renderer"]["pan_y"];
        assert_eq!(zoom.as_f64().unwrap(), 100.0);
        assert_eq!(pan_x.as_f64().unwrap(), 100.0);
        assert_eq!(pan_y.as_f64().unwrap(), 100.0);
    }
}
