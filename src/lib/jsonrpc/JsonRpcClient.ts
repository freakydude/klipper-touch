import { writable } from 'svelte/store';
import type { IJsonRpcErrorResponse } from './types/IJsonRpcErrorResponse';
import type { IJsonRpcRequest } from './types/IJsonRpcRequest';
import type { IJsonRpcResponse } from './types/IJsonRpcResponse';
import type { IJsonRpcSuccessResponse } from './types/IJsonRpcSuccessResponse';

export class JsonRpcClient extends EventTarget {
  private _ws?: WebSocket;
  private _url;
  private requestTimeout = 30 * 1000;

  public isConnected = writable(false);

  public constructor(url: string | URL) {
    super();
    this._url = url;
  }

  public connect(): Promise<boolean> {
    const result: Promise<boolean> = new Promise<boolean>((resolve, reject) => {
      if (this._ws != undefined) {
        const message = 'Websocket already initialized';
        console.error(message);
        reject(message);
      }

      try {
        this.isConnected.set(false);
        this._ws = new WebSocket(this._url);

        this._ws.onopen = (event: Event) => {
          console.log('Websocket opened ', event);
          this.isConnected.set(true);
          resolve(true);
        };

        this._ws.onclose = (event: CloseEvent) => {
          this.isConnected.set(false);
          console.log('Websocket closed ', event);
        };

        this._ws.onerror = (event: Event) => {
          this.isConnected.set(false);
          console.error('Websocket error ', event);
        };

        this._ws.onmessage = (event: MessageEvent) => {
          const request: IJsonRpcRequest | IJsonRpcRequest[] = JSON.parse(event.data);

          // console.log('connect ws.onmessage: single', request);
          if (Array.isArray(request)) {
            // batch request - send notification for every notification inside
            console.log('connect ws.onmessage: received batch request', request);
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
      } catch (error) {
        console.error(error);
        reject('Websocket could not be initialized: ${error}');
      }
    });

    return result;
  }

  public disconnect(): Promise<boolean> {
    const result: Promise<boolean> = new Promise<boolean>((resolve, reject) => {
      try {
        if (this.isConnected) {
          if (this._ws != undefined) {
            this._ws?.close();
            this._ws = undefined;
            this.isConnected.set(false);
          }

          resolve(true);
        } else {
          resolve(false);
        }
      } catch (error) {
        console.error(error);
        this.isConnected.set(false);
        reject('Websocket could not be disconnected: ${error}');
      }
    });

    return result;
  }

  public sendRequest(request: IJsonRpcRequest): Promise<IJsonRpcSuccessResponse | IJsonRpcErrorResponse> {
    let promise: Promise<IJsonRpcSuccessResponse | IJsonRpcErrorResponse>;

    if (!this.isConnected) {
      const notConnected = 'Websocket is not connected, send request failed';
      console.log(notConnected);
      promise = Promise.reject(notConnected);
    } else {
      promise = new Promise<IJsonRpcSuccessResponse | IJsonRpcErrorResponse>((resolve, reject) => {
        const timeout = setTimeout(() => {
          this._ws?.removeEventListener('message', parser);

          console.warn('Websocket Timeout send request: ', request);
          reject('Websocket Timeout send request');
        }, this.requestTimeout);

        const parser = (event: MessageEvent) => {
          const response: IJsonRpcSuccessResponse | IJsonRpcErrorResponse = JSON.parse(event.data);

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
        this._ws?.send(JSON.stringify(request));
      } catch (error) {
        console.log(error);
        promise = Promise.reject(error);
      }
    }

    return promise;
  }

  public sendBatchRequest(requests: IJsonRpcRequest[]): Promise<IJsonRpcSuccessResponse[] | IJsonRpcErrorResponse[]> {
    let promise: Promise<IJsonRpcSuccessResponse[] | IJsonRpcErrorResponse[]>;

    if (!this.isConnected) {
      const notConnected = 'Websocket is not connected, send batch request failed';
      console.log(notConnected);
      promise = Promise.reject(notConnected);
    } else {
      //console.log('Websocket send patch requests:', requests);

      promise = new Promise<IJsonRpcSuccessResponse[] | IJsonRpcErrorResponse[]>((resolve, reject) => {
        const timeout = setTimeout(() => {
          this._ws?.removeEventListener('message', parser);

          console.warn('Websocket Timeout send batch request: ', requests);
          reject('Websocket Timeout send batch request');
        }, this.requestTimeout);

        const parser = (event: MessageEvent) => {
          const responses: IJsonRpcSuccessResponse[] | IJsonRpcErrorResponse[] = JSON.parse(event.data);

          // TODO if request is a invalid json -> response is a single error json. code don't care about this right now
          if (Array.isArray(responses)) {
            console.log('Websocket received batch responses', responses);
            const responsesWithId: IJsonRpcResponse[] = new Array<IJsonRpcResponse>();

            for (const element of responses) {
              if (element.id) [responsesWithId.push(element)];
            }

            if (responsesWithId.length >= 1) {
              clearTimeout(timeout);
              this._ws?.removeEventListener('message', parser);
              console.log('Websocket got response for request id:', responsesWithId[0].id, ' Responses:', responsesWithId);
              resolve(responses);
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
        this._ws?.send(JSON.stringify(requests));
      } catch (error) {
        console.log(error);
        promise = Promise.reject(error);
      }
    }

    return promise;
  }

  public static generateConnectionId(): string {
    return Math.floor(Math.random() * 1000000).toString();
  }
}
