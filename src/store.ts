import { SvelteComponent } from 'svelte';
import { writable } from 'svelte/store';

export const currentScreen = writable(SvelteComponent);

// temperatures

export const nozzleTemp = writable(200.0);
export const heatbedTemp = writable(60.0);

// fans
export const nozzleFanSpeed = writable(90.0);

// nozzlePosition
export const nozzleX = writable(150.0);
export const nozzleY = writable(150.0);
export const nozzleZ = writable(50.0);
export const nozzleBabyZ = writable(0.02);
