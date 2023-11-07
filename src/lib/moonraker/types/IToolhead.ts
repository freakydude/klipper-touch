export interface IToolhead {
  position: number[];
  homed_axes: string;
  max_accel: number;
  axis_minimum: number[];
  axis_maximum: number[];
  max_velocity: number;
  square_corner_velocity: number;
  max_accel_to_decel: number;
}
