import { writable } from 'svelte/store';

export class GCodeMove {
  public HomeOrigin = writable([0.0, 0.0, 0.0, 0.0]);
  public Speed = writable(0.0);
  public SpeedFactor = writable(0.0);
  public ExtrudeFactor = writable(0.0);
}
