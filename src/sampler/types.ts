export const enum Expression {
  Natural = "natural",
  Pizzicato = "pizzicato",
  Spiccato = "spiccato",
  Staccato = "staccato",
  Tremolo = "tremolo",
  Mute = "mute",
}

export type Tick = number;
export type Ticks = number;
export type Seconds = number;
export type BPM = number;

/**
 * A callback to be used with the schedule function to schedule events at a specific tick on
 * the scheduler timeline. It will be called with 2 args:
 *
 * __at__: the time to fire the audio event using the highly accurate AudioContext time.
 *
 * __duration__: The duration in sconds taking into account tempo changes.
 *
 * ```
 * transport.schedule((at, duration) => {
 *      AudioBufferSourceNode.start(at, 0, duration);
 * }, 16, 16)
 * ```
 */
export type Callback = (when: Seconds, duration: Seconds) => void;

export type Progress = (total: number, complete: number) => void;

/**
 * [pitch, attack, release, start_loop, end_loop, data];
 */
export type Patch = [number, Seconds, Seconds, string];
