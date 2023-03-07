import { writable } from 'svelte/store';

export class GCodeMove {
  public HomeOrigin = writable(0.0);
}
