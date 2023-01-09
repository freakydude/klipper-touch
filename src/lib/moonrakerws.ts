export interface IMoonrakerWs {
  connect(): void;
  get isConnected(): Boolean;
  sendMessage(message: string): Promise<string>;
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
        console.log('onmessage -> id:' + data.id + ' content: ' + data.content);
      }
    };
  }

  public sendMessage(message: string): Promise<string> {
    let promise: Promise<string> = Promise.reject('Something went completely wrong');

    if (!this.isConnected) {
      console.log('Not connected');
      promise = Promise.reject('ws not connected');
    }

    if (this.ws.readyState <= 1) {
      console.log('Websocket sendMessage: ' + message);

      promise = new Promise<string>((resolve, reject) => {
        setTimeout(() => {
          this.ws.removeEventListener('message', parser);
          reject('Timeout');
        }, 30 * 1000);

        let parser = (event: MessageEvent) => {
          const data: Item = JSON.parse(event.data);
          if (this.connectionId.toString() == data.id) {
            if ('bla' == 'bla') {
              this.ws.removeEventListener('message', parser);
              resolve('dieAntwort');
            }
          }
        };

        this.ws.addEventListener('message', parser);
      });

      this.ws.send(message);
    } else {
      promise = Promise.reject('ws not ready');
    }

    return promise;
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
