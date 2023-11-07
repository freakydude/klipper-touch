import { writable } from 'svelte/store';

export class ConfigFile {
  public Config = writable({});
  public Settings = writable({});
  public SaveConfigPending = writable(false);
}
