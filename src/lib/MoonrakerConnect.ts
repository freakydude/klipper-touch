export class MoonrakerConnect {
  private connected: Boolean = false;
  private connectionId: string = '';
  private ws: WebSocket;

  public constructor(url: string | URL) {
    this.ws = new WebSocket(url);
  }

  public async connect(): Promise<boolean> {
    this.connected = false;
    this.connectionId = this.generateConnectionId();

    let result: Promise<boolean> = new Promise<boolean>((resolve, reject) => {
      if (!this.ws) {
        reject('no ws');
      }

      this.ws.onopen = (event: Event) => {
        console.log('Websocket opened - Message: ' + event.timeStamp);
        this.connected = true;

        this.requestIdentifyConnection()
          .then(() => resolve(true))
          .catch(() => reject('identify connection rejected'));
      };

      this.ws.onclose = (event: CloseEvent) => {
        console.log('Websocket closed - Message: ' + event.reason);
      };

      this.ws.onerror = (event: Event) => {
        console.log('Websocket error - Message: ' + event);
      };

      this.ws.onmessage = (event: MessageEvent) => {
        const data: any = JSON.parse(event.data);

        if (this.connectionId == data.id) {
          console.log('onmessage -> id:' + data.id + ' result: ' + data.result?.connection_id);
        }
      };
    });

    return result;
  }

  public async sendMessage(message: any): Promise<string> {
    let promise: Promise<string>;

    if (!this.connected) {
      console.log('ws not connected');
      promise = Promise.reject('Reject: ws not connected');
    } else if (this.ws.readyState <= 1) {
      console.log('Websocket sendMessage: ' + JSON.stringify(message));

      promise = new Promise<string>((resolve, reject) => {
        let timeout = setTimeout(() => {
          this.ws.removeEventListener('message', parser);

          console.log('sendMessage - Timeout reject');
          reject('sendMessage - Timeout');
        }, 30 * 1000);

        let parser = (event: MessageEvent) => {
          const data: any = JSON.parse(event.data);

          console.log('parser messageid: ' + message.id + ' dataid:' + data.id);

          if (message.id == data.id) {
            this.ws.removeEventListener('message', parser);

            console.log('parser die Antwort für id:' + data.id + ' result: ' + event.data);
            clearTimeout(timeout);
            resolve(data);
          }
        };

        console.log('send message');
        this.ws.addEventListener('message', parser);
      });

      this.ws.send(JSON.stringify(message));
    } else {
      console.log('ws not ready reject');
      promise = Promise.reject('ws not ready');
    }

    return promise;
  }

  public generateConnectionId(): string {
    return Math.floor(Math.random() * 10000).toString();
  }

  public async requestIdentifyConnection(): Promise<string> {
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

    let result = await this.sendMessage(identifyConnectionRequest);
    console.log('requestIdentifyConnection - isConnected');
    console.log(result);

    return result;
  }
}
