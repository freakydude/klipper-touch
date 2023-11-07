export interface IMotionReport {
  live_position: number[];
  live_velocity: number;
  live_extruder_velocity: number;
  steppers: string[];
  trapq: string[];
}
