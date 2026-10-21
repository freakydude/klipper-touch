import type { CliMatches } from '@tauri-apps/plugin-cli';
import { load, type Store } from '@tauri-apps/plugin-store';
import { get, writable, type Writable } from 'svelte/store';

export class BootParams {
  public moonrakerApi: Writable<URL> = writable(new URL('http://127.0.0.1'));
  public moonrakerWs: Writable<URL> = writable(new URL('ws://127.0.0.1/websocket'));
  public fullscreen: Writable<boolean> = writable(false);
  public store: Writable<Store> = writable();

  public async loadStore() {
    const store = await load('config.json', {
      autoSave: false,
      defaults: {
        moonrakerApi: get(this.moonrakerApi).href,
        moonrakerWs: get(this.moonrakerWs).href,
        fullscreen: get(this.fullscreen)
      }
    });
    const moonrakerApi = await store.get<unknown>('moonrakerApi');
    const moonrakerWs = await store.get<unknown>('moonrakerWs');
    const fullscreen = await store.get<unknown>('fullscreen');

    if (typeof moonrakerApi === 'string') {
      this.moonrakerApi.set(new URL(moonrakerApi));
    }
    if (typeof moonrakerWs === 'string') {
      this.moonrakerWs.set(new URL(moonrakerWs));
    }
    if (typeof fullscreen === 'boolean') {
      this.fullscreen.set(fullscreen);
    }
  }

  public setMatches(matches: CliMatches) {
    if (matches.args) {
      const args = matches.args;

      if (args.moonrakerapi?.value) {
        this.moonrakerApi.set(new URL(args.moonrakerapi.value as string));
      }
      if (args.moonrakerws?.value) {
        this.moonrakerWs.set(new URL(args.moonrakerws.value as string));
      }
      if (args.fullscreen?.value) {
        this.fullscreen.set(args.fullscreen.value as boolean);
      }
    }
  }
}
