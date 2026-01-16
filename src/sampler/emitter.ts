type Listener = (...args: unknown[]) => void;
interface Listeners {
  [event: string]: Set<Listener>;
}

/**
 * Super simple event emitter
 */
export class EventEmitter<T extends string> {
  private _events: Listeners = {};

  private _getEventListByName(eventName: T) {
    if (typeof this._events[eventName] === "undefined") {
      this._events[eventName] = new Set();
    }
    return this._events[eventName];
  }

  public on(event: T, fn: Listener) {
    this._getEventListByName(event).add(fn);
  }

  public emit(event: T, ...args: unknown[]) {
    this._getEventListByName(event).forEach((fn) => {
      fn.apply(this, args);
    });
  }

  public removeListener(event: T, fn: Listener) {
    this._getEventListByName(event).delete(fn);
  }
}
