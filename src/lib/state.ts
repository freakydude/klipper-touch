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

export const connect = (socketURL: string) => {
  const ws = new WebSocket(`ws://${socketURL}`);
  if (!ws) {
    state.update((s: State) => {
      return { ...s, error: 'Unable to connect' };
    });
    return;
  }

  ws.addEventListener('open', () => {
    console.log('Websocket opened');
  });

  ws.addEventListener('message', (message: any) => {
    console.log('Websocket message');

    const data: Item = JSON.parse(message.data);
    // Mutate state by prepending the new data to the array.
    state.update((state) => ({ ...state, items: [data].concat(state.items) }));
  });

  ws.addEventListener('close', (_message: any) => {
    console.log('Websocket closed');
  });

  const sendMessage = (message: any) => {
    if (ws.readyState <= 1) {
      ws.send(message);
    }
  };
};
