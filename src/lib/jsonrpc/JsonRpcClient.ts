import { writable } from 'svelte/store';
import type { IJsonRpcRequest } from './types/IJsonRpcRequest';
import type { IJsonRpcResponse } from './types/IJsonRpcResponse';

export class JsonRpcClient extends EventTarget {
  private _ws?: WebSocket;
  private _url: string | URL;
  private static _id: number = 0;
  private requestTimeout = 30 * 1000;

  public isConnected = writable(false);

  public constructor(url: string | URL) {
    super();
    this._url = url;
  }

  public connect(): Promise<boolean> {
    const result: Promise<boolean> = new Promise<boolean>((resolve, reject) => {
      try {
        if (!this._ws) {
          this.isConnected.set(false);
          console.log('JsonRpcClient.connect() Create new WebSocket');
          this._ws = new WebSocket(this._url);

          this._ws.onopen = (event: Event) => {
            console.log('JsonRpcClient.WebSocket.onopen ', event);
            this.isConnected.set(true);
            resolve(true);
          };

          this._ws.onclose = (event: CloseEvent) => {
            this.isConnected.set(false);
            this._ws = undefined;
            console.log('JsonRpcClient.WebSocket.onclose ', event);
          };

          this._ws.onerror = (event: Event) => {
            this.isConnected.set(false);
            this._ws = undefined;
            console.warn('JsonRpcClient.WebSocket.onerror ', event);
          };

          this._ws.onmessage = (event: MessageEvent) => {
            const request: IJsonRpcRequest | IJsonRpcRequest[] = JSON.parse(event.data);

            // console.log('connect ws.onmessage: single', request);
            if (Array.isArray(request)) {
              // batch request - send notification for every notification inside
              console.log('JsonRpcClient.WebSocket.onmessage received batch request', request);
              for (const singleRequest of request) {
                if (!singleRequest.id) {
                  // console.log('connect ws.onmessage: notification', request);
                  this.dispatchEvent(
                    new CustomEvent<IJsonRpcRequest[]>('notification', {
                      detail: request
                    })
                  );
                }
              }
            } else {
              // single request
              if (!request.id) {
                // console.log('connect ws.onmessage: received singleRequest', request);
                this.dispatchEvent(
                  new CustomEvent<IJsonRpcRequest>('notification', {
                    detail: request
                  })
                );
              }
            }
          };
        } else {
          console.log('JsonRpcClient.connect() WebSocket already initialized');
          resolve(true);
        }
      } catch (error) {
        console.error('JsonRpcClient.connect() ', error);
        reject('Websocket could not be initialized: ${error}');
      }
    });

    return result;
  }

  public disconnect(): Promise<boolean> {
    const result: Promise<boolean> = new Promise<boolean>((resolve, reject) => {
      try {
        if (this._ws) {
          this._ws.close();
          resolve(true);
        } else {
          console.warn('JsonRpcClient.disconnect() WS ws not OPEN');
          resolve(false);
        }
      } catch (error) {
        console.error('JsonRpcClient.disconnect() ', error);
        reject('Websocket could not be disconnected: ${error}');
      }
    });

    return result;
  }

  public sendRequest(request: IJsonRpcRequest): Promise<IJsonRpcResponse> {
    let promise: Promise<IJsonRpcResponse>;

    if (!this._ws?.OPEN) {
      const notConnected = 'Websocket is not connected, send request failed';
      console.error('JsonRpcClient.sendRequest() ', notConnected);
      promise = Promise.reject(notConnected);
    } else {
      promise = new Promise<IJsonRpcResponse>((resolve, reject) => {
        const timeout = setTimeout(() => {
          this._ws?.removeEventListener('message', parser);

          console.warn('JsonRpcClient.sendRequest() Timeout send request: ', request);
          reject('Websocket Timeout send request');
        }, this.requestTimeout);

        const parser = (event: MessageEvent) => {
          const response: IJsonRpcResponse = JSON.parse(event.data);

          if (request.id == response.id) {
            clearTimeout(timeout);
            this._ws?.removeEventListener('message', parser);

            console.log('Websocket got response for request id:', response.id, 'response:', response);

            resolve(response);
          }
          // else {
          //   console.log('parser some message - request: ', request, ' response:', response);
          // }
        };
        this._ws?.addEventListener('message', parser);
      });

      try {
        // console.log("request", request)
        this._ws.send(JSON.stringify(request));
      } catch (error) {
        console.log(error);
        promise = Promise.reject(error);
      }
    }

    return promise;
  }

  public sendBatchRequest(requests: IJsonRpcRequest[]): Promise<IJsonRpcResponse[]> {
    let promise: Promise<IJsonRpcResponse[]>;

    if (!this._ws?.OPEN) {
      const notConnected = 'Websocket is not connected, send batch request failed';
      console.log(notConnected);
      promise = Promise.reject(notConnected);
    } else {
      promise = new Promise<IJsonRpcResponse[]>((resolve, reject) => {
        const timeout = setTimeout(() => {
          this._ws?.removeEventListener('message', parser);

          console.warn('Websocket Timeout send batch request: ', requests);
          reject('Websocket Timeout send batch request');
        }, this.requestTimeout);

        const parser = (event: MessageEvent) => {
          const responses: IJsonRpcResponse[] = JSON.parse(event.data);

          // TODO if request is a invalid json -> response is a single error json. code don't care about this right now
          if (Array.isArray(responses)) {
            console.log('Websocket received batch responses', responses);
            const responsesWithId: IJsonRpcResponse[] = [...responses.filter((response) => response.id !== null)];

            if (responsesWithId.length) {
              clearTimeout(timeout);
              this._ws?.removeEventListener('message', parser);
              console.log('Websocket got response for request id:', responsesWithId[0].id, ' Responses:', responsesWithId);
              resolve(responsesWithId);
            }
            // else {
            //   console.log('parser some message - requests:', requests, ' responses:', responses);
            // }
          }
        };
        this._ws?.addEventListener('message', parser);
      });

      try {
        // console.log("request", request)
        this._ws.send(JSON.stringify(requests));
      } catch (error) {
        console.log(error);
        promise = Promise.reject(error);
      }
    }

    return promise;
  }

  public static generateConnectionId(): string {
    this._id++;

    return this._id.toString();
  }
}
