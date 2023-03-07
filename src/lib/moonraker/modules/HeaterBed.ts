import { writable } from 'svelte/store';

export class HeaterBed {
  public Target = writable(0.0);
  public Temperature = writable(0.0);
}
