import { writable } from 'svelte/store';

export class Toolhead {
  public Position = writable([0, 0, 0, 0]);
  public HomedAxes = writable('');
  public MaxAcceleration = writable(0.0);
}
