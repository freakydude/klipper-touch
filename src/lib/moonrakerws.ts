export interface IMoonrakerWs {
  connect(): void;
  get isConnected(): Boolean;
  sendMessage(message: string | ArrayBufferLike | Blob | ArrayBufferView): void;
}

type Item = {
  id: string;
  content: string;
};

export class MoonrakerWS implements IMoonrakerWs {
  url: URL;
  connected: Boolean = false;
  ws: WebSocket;
  connectionId: Number;

  public constructor(url: URL) {
    this.url = url;
    this.ws = new WebSocket(this.url);
    this.connectionId = Number.NaN;
  }

  public get isConnected(): Boolean {
    return this.connected;
  }

  public connect() {
    this.connected = false;
    this.ws = new WebSocket(`${import.meta.env.VITE_MOONRAKER_WEBSOCKET}`);
    this.connectionId = this.generateConnectionId();

    if (!this.ws) {
      return;
    }

    this.ws.onopen = (event: Event) => {
      console.log('Websocket opened - Message: ' + event);

      this.requestIdentifyConnection();
    };

    this.ws.onclose = (event: CloseEvent) => {
      console.log('Websocket closed - Message: ' + event.reason);
    };

    this.ws.onerror = (event: Event) => {
      console.log('Websocket error - Message: ' + event);
    };

    this.ws.onmessage = (event: MessageEvent) => {
      console.log('Websocket message - lastEventId:' + event.lastEventId + ' eventData: ' + event.data);

      const data: Item = JSON.parse(event.data);

      if (this.connectionId.toString() == data.id) {
        // Mutate state by prepending the new data to the array.
      }
    };
  }

  public sendMessage(message: string | ArrayBufferLike | Blob | ArrayBufferView) {
    if (!this.isConnected) {
      console.log('Not connected');
    }

    if (this.ws.readyState <= 1) {
      console.log('Websocket sendMessage: ' + message);

      this.ws.send(message);
    }
  }

  generateConnectionId(): Number {
    return Math.floor(Math.random() * 10000);
  }

  requestIdentifyConnection() {
    const identifyConnectionRequest = {
      jsonrpc: '2.0',
      method: 'server.connection.identify',
      params: {
        client_name: 'klipper-touch',
        version: '0.0.1',
        type: 'display',
        url: 'https://github.com/freakydude/klipper-touch'
      },
      id: this.connectionId
    };

    var message = JSON.stringify(identifyConnectionRequest);
    this.sendMessage(message);
  }
}
