import { writable } from 'svelte/store';

export class MotionReport {
  public LivePosition = writable([0.0, 0.0, 0.0, 0.0]);
  public LiveVelocity = writable(0.0);
  public LiveExtruderVelocity = writable(0.0);
  public Steppers = writable<string[]>([]);
  public Trapq = writable<string[]>([]);
}
