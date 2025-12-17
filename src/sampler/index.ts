import { AudioPlayer } from "./audio-player";
import { Scheduler } from "./scheduler";

export * from "./types";

const ctx = new AudioContext();
export const Transport = new Scheduler(ctx);
export const Player = new AudioPlayer(ctx);
