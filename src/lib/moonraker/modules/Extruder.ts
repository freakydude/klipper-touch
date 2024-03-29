import { writable } from 'svelte/store';

export class Extruder {
  public Temperature = writable(0.0);
  public Target = writable(0.0);
  public PressureAdvance = writable(0.0);
  public CanExtrude = writable(false);
}
