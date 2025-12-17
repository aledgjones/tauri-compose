import { EventEmitter } from "./emitter";
import { Tempo } from "./tempo";
import { BPM, Callback, Seconds, Tick, Ticks } from "./types";

/**
 * Schedule events along a predefined timeline of events.
 */
export class Scheduler extends EventEmitter<"start" | "stop" | "tick"> {
  constructor(ctx: AudioContext) {
    super();
    this.ctx = ctx;
  }

  /**
   * The audio context is used for precise timing of events.
   */
  public ctx: AudioContext;

  /**
   * Tempo events to be scheduled
   */
  private _tempo_events: { [tick: number]: Tempo } = {};

  /**
   * All non-tempo events to be scheduled.
   */
  private _callback_events: { [tick: number]: Set<[Ticks, Callback]> } = {};

  /**
   * The current tick position of the scheduler.
   */
  private _tick: Tick = 0;

  public get tick() {
    return this._tick;
  }

  public set tick(tick: Tick) {
    this._tick = tick;
    this.emit("tick", tick);
  }

  private _subdivisions: number = 16;

  /**
   * Subdivisions are a division of the quarter beat. They are the smallest time value the
   * scheduler understands and represent one 'tick' of the scheduler.
   */
  public get subdivisions() {
    return this._subdivisions;
  }

  public set subdivisions(value: number) {
    this._subdivisions = value;
  }

  private _length: Ticks = 16;

  /**
   * The duration in ticks of the whole schecduler timeline. This means we can calculate the duration
   * of each tick (and therefore tempo changes) in advance.
   */
  public get length() {
    return this._length;
  }

  public set length(value: number) {
    this._length = value;
    this._parseTempoChanges();
  }

  /**
   * For each tick we calculate its length in seconds based on the tempo at the given tick.
   * This is stored here.
   */
  private _tickOffsets: Seconds[] = [];

  /**
   * Whenever we create tempo changes we need to update the tick durations.
   */
  private _parseTempoChanges() {
    // default to 120
    let tempo = new Tempo(0, 120);

    let currentTime = 0;
    for (let tick = 0; tick <= this.length; tick++) {
      if (this._tempo_events[tick]) {
        tempo = this._tempo_events[tick];
      }

      this._tickOffsets[tick] = currentTime;

      const currentTempo = tempo.getAt(tick);
      currentTime = currentTime + 60 / currentTempo / this.subdivisions;
    }
  }

  private _playing: boolean = false;
  private _playStartTime: number = 0;
  private _scheduledTicks: { [tick: number]: boolean } = {};

  private _schedulePlayback(tick: Tick, when: Seconds) {
    const events = this._callback_events[tick];
    if (events) {
      events.forEach(([duration, callback]) => {
        // calculate how long the duration is in seconds.
        const span =
          this._tickOffsets[tick + duration] - this._tickOffsets[tick];
        callback(when, span);
      });
    }
  }

  /**
   * Get the current tick during playback
   */
  private getCurrentTick() {
    const position = this.ctx.currentTime - this._playStartTime;
    for (let tick = 0; tick <= this.length; tick++) {
      if (this._tickOffsets[tick] > position) {
        return tick < 0 ? 0 : tick - 1;
      }
    }

    return this.length;
  }

  private loop() {
    const tick = this.getCurrentTick();
    if (tick !== this.tick) {
      this.tick = tick;
    }
    if (tick === this.length) {
      this.pause();
    }
    const lookaheadTime = this.ctx.currentTime - this._playStartTime + 0.2;
    for (let tick = this.tick; tick <= this._length; tick++) {
      // only look ahead to events less than 200ms away
      if (this._tickOffsets[tick] < lookaheadTime) {
        // only schedule events not already scheduled
        if (this._scheduledTicks[tick] === undefined) {
          // scedule events
          this._schedulePlayback(
            tick,
            this._playStartTime + this._tickOffsets[tick],
          );
          // set the tick as scheduled
          this._scheduledTicks[tick] = true;
        }
      } else {
        break;
      }
    }

    requestAnimationFrame(() => {
      if (this._playing) {
        this.loop.call(this);
      }
    });
  }

  /**
   * Remove all scheduled events from the scheduler. This includes tempo events and callback
   * events.
   */
  public clear() {
    this._tempo_events = {};
    this._callback_events = {};
    this._tickOffsets = [];

    // after clearing all the events we need to recalculate the tick track
    this._parseTempoChanges();
  }

  /**
   * Schedule a tempo change at a given tick.
   *
   * If _from_ and _duration_ args are omitted, the tempo will take effect immediately. Else the
   * tempo will be ramped smoothly between the to and from values over the duration given
   */
  public scheduleTempoChange(
    tick: Tick,
    to: BPM,
    from?: BPM,
    duration?: Ticks,
  ) {
    this._tempo_events[tick] = new Tempo(tick, to, from, duration);
    this._parseTempoChanges();
  }

  public scheduleEvent(tick: Tick, duration: Ticks, callback: Callback) {
    if (!this._callback_events[tick]) {
      this._callback_events[tick] = new Set();
    }

    // store the callback and duration in ticks as tempos may be added after
    // these need to be calculated on the fly.
    this._callback_events[tick].add([duration, callback]);
  }

  public seek(tick: Tick) {
    if (this._playing) {
      this.pause(true);
      this.tick = tick;
      this.start(true);
    } else {
      this.tick = tick;
    }
  }

  public start(silent?: boolean) {
    if (this.tick === this.length) {
      return;
    }

    if (!silent) {
      this.emit("start");
    }
    this._playing = true;
    // we work out when tick 0 would have been.
    this._playStartTime = this.ctx.currentTime - this._tickOffsets[this.tick];

    this.loop();
  }

  public stop(silent?: boolean) {
    this.pause(silent);
    this.tick = 0;
  }

  public pause(silent?: boolean) {
    if (!silent) {
      this.emit("stop");
    }
    this._playing = false;
    this._scheduledTicks = {};
  }
}
