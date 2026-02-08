import type { CliMatches } from '@tauri-apps/plugin-cli';
import { load, Store } from '@tauri-apps/plugin-store';
import { writable, type Writable } from 'svelte/store';

export class BootParams {
  public moonrakerApi: Writable<URL> = writable(new URL('http://127.0.0.1'));
  public moonrakerWs: Writable<URL> = writable(new URL('ws://127.0.0.1/websocket'));
  public fullscreen: Writable<boolean> = writable(false);
  public store: Writable<Store> = writable();

  public async loadStore() {
    const store = await load('config.json', {
      autoSave: false,
      defaults: { moonrakerApi: this.moonrakerApi, moonrakerWs: this.moonrakerWs, fullscreen: this.fullscreen }
    });
    const moonrakerApi = await store.get<string>('moonrakerApi');
    const moonrakerWs = await store.get<string>('moonrakerWs');
    const fullscreen = await store.get<boolean>('fullscreen');

    if (moonrakerApi) {
      this.moonrakerApi.set(new URL(moonrakerApi));
    }
    if (moonrakerWs) {
      this.moonrakerWs.set(new URL(moonrakerWs));
    }
    if (fullscreen !== undefined) {
      this.fullscreen.set(fullscreen);
    }
  }

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
