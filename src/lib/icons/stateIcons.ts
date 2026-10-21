import type { IconName } from './icon-names';

const printStateIcons: Record<string, IconName> = {
  standby: 'state-standby',
  printing: 'state-printing',
  paused: 'state-paused',
  complete: 'state-complete',
  cancelled: 'state-cancelled',
  error: 'state-error'
};

const klippyStateIcons: Record<string, IconName> = {
  disconnected: 'state-disconnected',
  startup: 'state-startup',
  ready: 'state-ready',
  shutdown: 'state-shutdown',
  error: 'state-error'
};

/** Icon zum Druckzustand (`print_stats.state`); unbekannte Zustände zeigen das Standby-Icon. */
export function printStateIcon(state: string): IconName {
  return printStateIcons[state] ?? 'state-standby';
}

/** Icon zum Klippy-Zustand (`webhooks.state`); unbekannte Zustände zeigen das Fehler-Icon. */
export function klippyStateIcon(state: string): IconName {
  return klippyStateIcons[state] ?? 'state-error';
}
