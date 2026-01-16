import { invoke } from "@tauri-apps/api/core";
import { UniqueId } from "./types";

export interface HitTestResult {
  tone_id: UniqueId;
  track_id: UniqueId;
  flow_id: UniqueId;
  instrument_id: UniqueId;
}

export async function hitTest(
  x: number,
  y: number,
  panX: number,
  panY: number,
  zoom: number,
): Promise<HitTestResult | null> {
  return await invoke<HitTestResult | null>("renderer_hit_test", {
    x,
    y,
    pan_x: panX,
    pan_y: panY,
    zoom,
  });
}
