import { writable } from 'svelte/store';

export class Toolhead {
  public Position = writable([0, 0, 0, 0]);
  public HomedAxes = writable('');
  public MaxAcceleration = writable(0.0);
  public MaxDeceleration = writable(0.0);
  public MaxVelocity = writable(0.0);
  public SquareCornerVelocity = writable(0.0);
  public AxisMinimum = writable([0, 0, 0, 0]);
  public AxisMaximum = writable([0, 0, 0, 0]);
}
