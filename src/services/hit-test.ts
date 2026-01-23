import { invoke } from "@tauri-apps/api/core";
import { Selection } from "./types";

export async function hitTest(
  x: number,
  y: number,
  panX: number,
  panY: number,
  zoom: number,
) {
  return await invoke<Selection | null>("renderer_hit_test", {
    x,
    y,
    pan_x: panX,
    pan_y: panY,
    zoom,
  });
}
