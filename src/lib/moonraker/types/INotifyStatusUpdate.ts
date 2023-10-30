import type { TKlippyState } from './TKlippyState';
import type { TPrintState } from './TPrintState';

export interface IWebhooks {
  state: TKlippyState;
  state_message: string;
}

export interface IHeaterBed {
  temperature: number;
  target: number;
}

export interface IExtruder {
  temperature: number;
  target: number;
  pressure_advance: number;
}

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

export interface IMotionReport {
  live_position: number[];
  live_velocity: number;
  live_extruder_velocity: number;
  steppers: string[];
  trapq: string[];
}

export interface IFan {
  speed: number;
  rpm: number;
}

export interface IGCodeMove {
  homing_origin: number[];
  speed: number;
  speed_factor: number;
  extrude_factor: number;
}

export interface IPrintStats {
  filename: string;
  total_duration: number;
  print_duration: number;
  filament_used: number;
  state: TPrintState;
  message: string;
  info: IInfo;
}

export interface IInfo {
  current_layer: number;
  total_layer: number;
}

export interface IProgress {
  progress: number;
  message: string;
}

export interface INotifyStatusUpdateParams {
  webhooks: IWebhooks;
  heater_bed: IHeaterBed;
  extruder: IExtruder;
  toolhead: IToolhead;
  fan: IFan;
  gcode_move: IGCodeMove;
  print_stats: IPrintStats;
  display_status: IProgress;
  motion_report: IMotionReport;
}

export interface INotifyStatusUpdate {
  params: INotifyStatusUpdateParams[];
}
