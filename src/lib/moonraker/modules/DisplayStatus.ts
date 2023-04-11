import { writable } from 'svelte/store';

export class DisplayStatus {
  public Progress = writable(0.0);
}
