import type { MoonrakerClient } from '$lib/moonraker/MoonrakerClient';
import { readable, type Readable } from 'svelte/store';

export class Values {
  private _moonrakerClient: MoonrakerClient;

  public clockFormatter: Intl.DateTimeFormat;
  public clock: Readable<Date>;

  public constructor(moonrakerClient: MoonrakerClient) {
    // super();
    this._moonrakerClient = moonrakerClient;
    this.clockFormatter = this.createClockFormatter();
    this.clock = this.createClock();
  }

  private createClockFormatter(): Intl.DateTimeFormat {
    return new Intl.DateTimeFormat('de', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  private createClock(): Readable<Date> {
    return readable(new Date(), function start(set) {
      const interval = setInterval(() => {
        set(new Date());
      }, 1000);

      return function stop() {
        clearInterval(interval);
      };
    });
  }
}
