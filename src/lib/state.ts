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

var ws: WebSocket;
var connId: number;

export function sendMessage(message: string | ArrayBufferLike | Blob | ArrayBufferView) {
  if (!ws) {
    console.log('sendMessage, but no WebSocket');
  }

  if (ws.readyState <= 1) {
    console.log('Websocket sendMessage' + message);
    ws.send(message);
  }
}

export function connect() {
  ws = new WebSocket(`${import.meta.env.VITE_MOONRAKER_WEBSOCKET}`);
  connId = generateConnectionId();

  if (!ws) {
    state.update((s: State) => {
      return { ...s, error: 'Unable to connect' };
    });
    return;
  }

  ws.onopen = (event: Event) => {
    console.log('Websocket opened - Message: ' + event);

    requestIdentifyConnection();
  };

  ws.onclose = (event: CloseEvent) => {
    console.log('Websocket closed - Message: ' + event.reason);
  };

  ws.onerror = (event: Event) => {
    console.log('Websocket error - Message: ' + event);
  };

  ws.onmessage = (event: MessageEvent) => {
    console.log('Websocket message - lastEventId:' + event.lastEventId + ' eventData: ' + event.data);

    const data: Item = JSON.parse(event.data);

    if (connId.toString() == data.id) {
      // Mutate state by prepending the new data to the array.
      state.update((state) => ({ ...state, items: [data].concat(state.items) }));
    }
  };

  function generateConnectionId(): number {
    return Math.floor(Math.random() * 10000);
  }

  function requestIdentifyConnection() {
    const identifyConnectionRequest = {
      jsonrpc: '2.0',
      method: 'server.connection.identify',
      params: {
        client_name: 'klipper-touch',
        version: '0.0.1',
        type: 'display',
        url: 'https://github.com/freakydude/klipper-touch'
      },
      id: connId
    };

    var message = JSON.stringify(identifyConnectionRequest);
    sendMessage(message);
  }
}
