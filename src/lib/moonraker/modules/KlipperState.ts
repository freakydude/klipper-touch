import { writable } from 'svelte/store';
import type { TKlippyState } from '../types/TKlippyState';

export class KlipperState {
  public state = writable<TKlippyState>('disconnected');
  public message = writable('');
}
