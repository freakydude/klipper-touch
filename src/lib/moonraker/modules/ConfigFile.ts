import { writable } from 'svelte/store';

export class ConfigFile {
  public SaveConfigPending = writable(false);
}
