import { InstrumentPlayer } from "./instrument-player";
import { Patch, Progress, Seconds } from "./types";
import { chain } from "./utils";

interface Sample {
  pitch: number;
  attack: Seconds;
  release: Seconds;
  source: AudioBuffer;
}

/**
 * A single group of samples. Each sample is assigned to a pitch.
 */
export class ExpressionPlayer {
  public ctx: AudioContext;
  public player: InstrumentPlayer;

  constructor(ctx: AudioContext, instrumentPlayer: InstrumentPlayer) {
    this.ctx = ctx;
    this.player = instrumentPlayer;
  }

  /**
   * Each event has a unique id which is just a incremented number so we can cancel them.
   */
  private inc = 0;
  private events: Map<number, [GainNode, AudioBufferSourceNode]> = new Map();

  /**
   * The audio buffers actually used to produce pitches.
   */
  private samples: Sample[] = [];

  /**
   * Returns the closest pitch to the requested pitch
   */
  private findClosestSampleToPitch(pitch: number): Sample {
    let closest: Sample | undefined;
    for (const entry of this.samples) {
      if (
        closest === undefined ||
        Math.abs(pitch - entry.pitch) < Math.abs(pitch - closest.pitch)
      ) {
        closest = entry;
      }
    }
    return closest || this.samples[0];
  }

  /**
   * Load a sample with a given pitch
   */
  public async load(url: string, progress: Progress) {
    const resp = await fetch(url);
    const patch: Patch[] = await resp.json();

    let complete = 0;
    return Promise.all(
      patch.map(async ([pitch, attack, release, data]) => {
        // get the sample as a buffer and decode
        const resp = await fetch(data);
        const buffer = await resp.arrayBuffer();
        const source = await this.ctx.decodeAudioData(buffer);

        this.samples.push({
          pitch,
          attack,
          release,
          source,
        });
        progress(patch.length, ++complete);
      }),
    );
  }

  /**
   * Play a pitch for a given duration at a specified time. It will find the buffer with
   * the closest pitch and tune it accordingly
   */
  public play(pitch: number, when: Seconds, duration: Seconds) {
    const id = this.inc++;
    const sample = this.findClosestSampleToPitch(pitch);

    const envelope = this.ctx.createGain();
    envelope.gain.value = 1.0;
    envelope.gain.setValueAtTime(1.0, when + duration);
    envelope.gain.exponentialRampToValueAtTime(
      0.01,
      when + duration + sample.release,
    );

    // create source and connect it to the base of the node chain
    const source = this.ctx.createBufferSource();
    source.buffer = sample.source;
    source.detune.value = (pitch - sample.pitch) * 100;
    chain(source, envelope, this.player.volumeNode);

    source.start(when);
    source.stop(when + duration + sample.release);

    this.events.set(id, [envelope, source]);

    source.onended = () => {
      this.events.delete(id);
      source.disconnect();
      envelope.disconnect();
    };
  }

  /**
   * Imediately stops all scheduled and playing sounds
   */
  public stopAll() {
    this.events.forEach((value, key) => {
      this.events.delete(key);
      value.forEach((node) => {
        node.disconnect();
      });
    });
  }

  /**
   * Disconnect all the playback nodes so they can be garbage collected
   */
  public disconnectAll() {
    this.stopAll();
    this.samples = [];
  }
}
