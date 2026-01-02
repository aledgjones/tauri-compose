import { InstrumentPlayer } from "./instrument-player";
import { Expression, Seconds } from "./types";

export class AudioPlayer {
  public ctx: AudioContext;
  private instruments: { [key: string]: InstrumentPlayer } = {};

  constructor(ctx: AudioContext) {
    this.ctx = ctx;
  }

  /**
   * Deal with muting and unmuting the MuteNode depending on overall state.
   *
   * Muting is dependant on solo instruments so it must take into account the whole state
   */
  public setSamplerMuteStates() {
    const keys = Object.keys(this.instruments);

    // solo trumps mute so we need to find if we have solos
    let found_solo = false;
    for (let i = 0; i < keys.length; i++) {
      const instrument = this.instruments[keys[i]];
      if (instrument.solo) {
        found_solo = true;
        break;
      }
    }

    // set the correct state
    keys.forEach((key) => {
      const instrument = this.instruments[key];
      if (instrument.solo) {
        instrument.muteNode.gain.value = 1;
      } else if (found_solo || instrument.mute) {
        instrument.muteNode.gain.value = 0;
      } else {
        instrument.muteNode.gain.value = 1;
      }
    });
  }

  public RMS(instrument: string) {
    return this.instruments[instrument]?.RMS() || 0;
  }

  public peak(instrument: string) {
    return this.instruments[instrument]?.peak() || 0;
  }

  public hasInstrument(key: string): boolean {
    return this.instruments[key] !== undefined;
  }

  /**
   * Create a new sampler controled by the audio player, or return existing one
   */
  public createSampler(key: string): InstrumentPlayer {
    const sampler = new InstrumentPlayer(this.ctx);
    this.instruments[key] = sampler;
    this.setSamplerMuteStates();
    return sampler;
  }

  /**
   * Play a note
   */
  public play(
    instrument: string,
    expression: Expression,
    pitch: number,
    when: Seconds,
    duration: Seconds,
  ) {
    if (!this.instruments[instrument]) {
      console.warn(
        `Attempted to play note for unknown instrument: ${instrument}`,
      );
      return;
    }
    this.instruments[instrument].play(expression, pitch, when, duration);
  }

  /**
   * Imediately stop all scheduled and playing sounds
   */
  public stopAll() {
    const keys = Object.keys(this.instruments);
    keys.forEach((key) => {
      this.instruments[key].stopAll();
    });
  }

  /**
   * Mute an instrument.
   *
   * This will take into account any solo instruments.
   */
  public mute(instrument: string) {
    if (!this.instruments[instrument]) {
      console.warn(`Attempted to mute unknown instrument: ${instrument}`);
      return;
    }
    this.instruments[instrument].mute = true;
    this.setSamplerMuteStates();
  }

  /**
   * Unmute an instrument.
   *
   * This will take into account any solo instruments.
   */
  public unmute(instrument: string) {
    if (!this.instruments[instrument]) {
      console.warn(`Attempted to unmute unknown instrument: ${instrument}`);
      return;
    }
    this.instruments[instrument].mute = false;
    this.setSamplerMuteStates();
  }

  /**
   * Solo an instrument.
   *
   * This will take into account any muted instruments.
   */
  public solo(instrument: string) {
    if (!this.instruments[instrument]) {
      console.warn(`Attempted to solo unknown instrument: ${instrument}`);
      return;
    }
    this.instruments[instrument].solo = true;
    this.setSamplerMuteStates();
  }

  /**
   * Unsolo an instrument.
   *
   * This will take into account any muted instruments.
   */
  public unsolo(instrument: string) {
    if (!this.instruments[instrument]) {
      console.warn(`Attempted to unsolo unknown instrument: ${instrument}`);
      return;
    }
    this.instruments[instrument].solo = false;
    this.setSamplerMuteStates();
  }

  /**
   * Set the volume of an instrument (0.0 - 1.0);
   */
  public volume(instrument: string, value: number) {
    if (!this.instruments[instrument]) {
      console.warn(
        `Attempted to set volume for unknown instrument: ${instrument}`,
      );
      return;
    }
    this.instruments[instrument].volume(value);
  }

  /**
   * Disconnect all the playback nodes so they can be garbage collected
   */
  public disconnect(key: string) {
    if (this.instruments[key]) {
      this.instruments[key].disconnectAll();
      delete this.instruments[key];
    }
  }
}
