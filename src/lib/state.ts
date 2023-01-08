import { writable } from 'svelte/store';

type Item = {
  id: string;
  content: string;
};

type State = {
  items: Array<Item>;
  error?: string;
};

export const state = writable<State>({
  items: []
});


