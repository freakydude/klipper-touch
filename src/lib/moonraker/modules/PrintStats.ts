import { writable } from 'svelte/store';
import type { TPrintState } from '../types/TPrintState';

export class PrintStats {
  public State = writable<TPrintState>('standby');
  public Message = writable('');
  public Filename = writable('');
  public PrintDuration = writable(0.0);
}
