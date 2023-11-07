import { IInfo } from './IInfo';
import type { TPrintState } from './TPrintState';

export interface IPrintStats {
  filename: string;
  total_duration: number;
  print_duration: number;
  filament_used: number;
  state: TPrintState;
  message: string;
  info: IInfo;
}
