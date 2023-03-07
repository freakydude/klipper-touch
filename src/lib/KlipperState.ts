import { writable } from 'svelte/store';
import { TKlippyState } from './moonraker-types/TKlippyState';

export class KlipperState {
  public klippyState = writable<TKlippyState>('disconnected');
  public klippyStateMessage = writable('');
}
