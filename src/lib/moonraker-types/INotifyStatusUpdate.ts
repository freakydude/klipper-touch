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
}

export interface IToolhead {
  position: number[];
  homed_axes: string;
}

export interface IFan {
  speed: number;
}

export interface IGCodeMove {
  homing_origin: number[];
}

export interface IPrintStats {
  filename: string;
  total_duration: number;
  print_duration: number;
  filament_used: number;
  state: TPrintState;
  message: string;
}

export interface IProgress {
  progress: number;
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
}

export interface INotifyStatusUpdate {
  params: INotifyStatusUpdateParams[];
}
