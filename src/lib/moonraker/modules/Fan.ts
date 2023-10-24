import { writable } from 'svelte/store';

export class Fan {
  public Speed = writable(0.0);
  public Rpm = writable(0.0);
}
