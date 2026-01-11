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
