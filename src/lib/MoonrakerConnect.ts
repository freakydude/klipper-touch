let connected: Boolean = false;
let ws: WebSocket;
let connectionId: string;

export async function connect(): Promise<boolean> {
  connected = false;
  ws = new WebSocket(import.meta.env.VITE_MOONRAKER_WEBSOCKET);
  connectionId = generateConnectionId();
  let result: Promise<boolean> = new Promise<boolean>((resolve, reject) => {
    if (!ws) {
      reject('no ws');
    }

    ws.onopen = (event: Event) => {
      console.log('Websocket opened - Message: ' + event.timeStamp);
      connected = true;

      requestIdentifyConnection()
        .then(() => resolve(true))
        .catch(() => reject('identify connection rejected'));
    };

    ws.onclose = (event: CloseEvent) => {
      console.log('Websocket closed - Message: ' + event.reason);
    };

    ws.onerror = (event: Event) => {
      console.log('Websocket error - Message: ' + event);
    };

    ws.onmessage = (event: MessageEvent) => {
      const data: any = JSON.parse(event.data);

      if (connectionId == data.id) {
        console.log('onmessage -> id:' + data.id + ' result: ' + data.result?.connection_id);
      }
    };
  });

  return result;
}

export async function sendMessage(message: any): Promise<string> {
  let promise: Promise<string>;

  if (!connected) {
    console.log('ws not connected');
    promise = Promise.reject('Reject: ws not connected');
  } else if (ws.readyState <= 1) {
    console.log('Websocket sendMessage: ' + JSON.stringify(message));

    promise = new Promise<string>((resolve, reject) => {
      let timeout = setTimeout(() => {
        ws.removeEventListener('message', parser);

        console.log('sendMessage - Timeout reject');
        reject('sendMessage - Timeout');
      }, 30 * 1000);

      let parser = (event: MessageEvent) => {
        const data: any = JSON.parse(event.data);

        console.log('parser messageid: ' + message.id + ' dataid:' + data.id);

        if (message.id == data.id) {
          ws.removeEventListener('message', parser);

          console.log('parser die Antwort für id:' + data.id + ' result: ' + event.data);
          clearTimeout(timeout);
          resolve(data);
        }
      };

      console.log('send message');
      ws.addEventListener('message', parser);
    });

    ws.send(JSON.stringify(message));
  } else {
    console.log('ws not ready reject');
    promise = Promise.reject('ws not ready');
  }

  return promise;
}

export function generateConnectionId(): string {
  return Math.floor(Math.random() * 10000).toString();
}

async function requestIdentifyConnection(): Promise<string> {
  const identifyConnectionRequest = {
    jsonrpc: '2.0',
    method: 'server.connection.identify',
    params: {
      client_name: 'klipper-touch',
      version: '0.0.1',
      type: 'display',
      url: 'https://github.com/freakydude/klipper-touch'
    },
    id: connectionId
  };

  let result = await sendMessage(identifyConnectionRequest);
  console.log('requestIdentifyConnection - isConnected');
  console.log(result);

  return result;
}
