import type { CliMatches } from '@tauri-apps/api/cli';
import { writable, type Writable } from 'svelte/store';

export class BootParams {
  public moonrakerApi: Writable<URL> = writable(new URL('http://127.0.0.1'));
  public moonrakerWs: Writable<URL> = writable(new URL('ws://127.0.0.1/websocket'));
  public fullscreen: Writable<boolean> = writable(false);

  public setMatches(matches: CliMatches) {
    if (matches.args !== undefined) {
      const args = matches.args;

      if (args.moonrakerapi?.occurrences) {
        this.moonrakerApi.set(new URL(args.moonrakerapi.value as string));
      }
      if (args.moonrakerws?.occurrences) {
        this.moonrakerWs.set(new URL(args.moonrakerws.value as string));
      }
      if (args.fullscreen?.occurrences) {
        this.fullscreen.set(args.fullscreen.value as boolean);
      }
    }
  }
}
