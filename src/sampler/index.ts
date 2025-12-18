import { AudioPlayer } from "./audio-player";
import { Scheduler } from "./scheduler";

export * from "./types";

export const ctx = new AudioContext();
export const transport = new Scheduler(ctx);
export const sampler = new AudioPlayer(ctx);
