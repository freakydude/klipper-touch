import { JsonRpcClient, JsonRpcRequest, type IJsonRpcSuccessResponse, type IJsonRpcErrorResponse, type IJsonRpcRequest } from '$lib/JsonRpcClient';
import { writable, type Readable } from 'svelte/store';

export class MoonrakerRpcClient extends EventTarget {
  jsonRpcClient: JsonRpcClient;
  _isReady = writable(false);

  public constructor(jsonRpcClient: JsonRpcClient) {
    super();
    this.jsonRpcClient = jsonRpcClient;

    this.attachToEvents();
  }

  public get isReady(): Readable<boolean> {
    return this._isReady as Readable<boolean>;
  }

  public async requestIdentifyConnection(): Promise<IJsonRpcSuccessResponse | IJsonRpcErrorResponse> {
    const identifyConnectionRequest: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: 'server.connection.identify',
      params: {
        client_name: 'klipper-touch',
        version: '0.0.1',
        type: 'display',
        url: 'https://github.com/freakydude/klipper-touch'
      },
      id: JsonRpcClient.generateConnectionId()
    };

    let result = await this.jsonRpcClient.sendRequest(identifyConnectionRequest);
    // console.log('requestIdentifyConnection - isConnected', result);

    return result;
  }

  public async connect() {
    while (true) {
      this._isReady.set(false);
      try {
        if (await this.jsonRpcClient.connect()) {
          let klippyState: string = '';
          while (true) {
            let serverInfoRequest = new JsonRpcRequest('server.info', JsonRpcClient.generateConnectionId(), undefined);

            try {
              let serverInfoResponse = (await this.jsonRpcClient.sendRequest(serverInfoRequest)) as IJsonRpcSuccessResponse;
              klippyState = serverInfoResponse.result.klippy_state;
              console.log('klippy_state: ', klippyState);

              if (klippyState == 'ready') {
                break;
              } else {
                await new Promise<void>((resolve) => {
                  setTimeout(() => {
                    resolve();
                  }, 1000 * 5);
                });
              }
            } catch (error) {
              console.log('MoonrakerRpcClient connect error: ', error);
            }
          }
          if (klippyState == 'ready') {
            this._isReady.set(true);

            break;
          }
        }
      } catch (error) {
        console.log('MoonrakerRpcClient connect error: ', error);
      }
    }

    this._isReady.set(true);
  }

  public async disconnect() {
    try {
      await this.jsonRpcClient.disconnect();
    } catch (error) {
      console.log('MoonrakerRpcClient disconnect error: ', error);
    }
    this._isReady.set(false);
  }

  protected attachToEvents() {
    this.jsonRpcClient.addEventListener('notification', (event: Event) => {
      this.parseNotification(event as CustomEvent<IJsonRpcRequest>);
    });

    this.jsonRpcClient.addEventListener('isConnected', (event: Event) => {
      if ((event as CustomEvent<boolean>).detail == false) {
        //  this.reconnect(event as CustomEvent<boolean>);
        this._isReady.set(false);
      }
    });
  }

  protected async reconnect(event: CustomEvent<boolean>): Promise<void> {
    if ((event as CustomEvent<boolean>).detail == false) {
      await this.connect();
    }
  }

  private async parseNotification(event: CustomEvent<IJsonRpcRequest>): Promise<void> {
    let notification = event.detail;

    // console.log('parseNotification: ', notification.method);

    switch (notification.method) {
      case 'notify_klippy_ready':
        this._isReady.set(true);
        break;
      case 'notify_klippy_disconnected ':
        console.log('notify_klippy_disconnected: ');

        try {
          await this.disconnect();
        } catch (error) {
          console.log(error);
        }
        try {
          await this.connect();
        } catch (error) {
          console.log(error);
        }

        break;
    }
  }
}
