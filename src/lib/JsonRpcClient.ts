export interface IJsonRpcRequest {
  jsonrpc: string;
  method: string;
  params?: any[] | object;
  id?: string | number | null;
}

export class JsonRpcRequest implements IJsonRpcRequest {
  jsonrpc: string;
  id?: string | number | null;
  method: string;
  params?: object | any[];

  /**
   *
   * @param jsonrpc A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
   * @param id An identifier established by the Client that MUST contain a String, Number, or NULL value if included. If it is not included it is assumed to be a notification. The value SHOULD normally not be Null and Numbers SHOULD NOT contain fractional parts.
   * @param method A String containing the name of the method to be invoked. Method names that begin with the word rpc followed by a period character (U+002E or ASCII 46) are reserved for rpc-internal methods and extensions and MUST NOT be used for anything else.
   * @param params A Structured value that holds the parameter values to be used during the invocation of the method. This member MAY be omitted.
   */
  public constructor(method: string, id?: string | number | null, params?: any[] | object) {
    this.jsonrpc = '2.0';
    this.id = id;
    this.method = method;
    this.params = params;
  }
}

export interface IJsonRpcResponse {
  jsonrpc: string;
  id?: string | number | null;
}

export interface IJsonRpcSuccessResponse extends IJsonRpcResponse {
  result: any;
}

export class JsonRpcSuccessResponse implements IJsonRpcSuccessResponse {
  result: any;
  jsonrpc: string;
  id?: string | number | null;

  /**
   *
   * @param jsonrpc A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
   * @param id This member is REQUIRED. It MUST be the same as the value of the id member in the Request Object. If there was an error in detecting the id in the Request object (e.g. Parse error/Invalid Request), it MUST be Null.
   * @param result This member is REQUIRED on success. This member MUST NOT exist if there was an error invoking the method. The value of this member is determined by the method invoked on the Server.
   */
  constructor(result: any, id?: string | number | null) {
    this.jsonrpc = '2.0';
    this.id = id;
    this.result = result;
  }
}

export interface IJsonRpcErrorObject {
  code: number;
  message: string;
  data?: object;
}

export class JsonRpcErrorObject implements IJsonRpcErrorObject {
  code: number;
  message: string;
  data?: object;

  /**
   *
   * @param code A Number that indicates the error type that occurred. This MUST be an integer.
   * @param message A String providing a short description of the error. The message SHOULD be limited to a concise single sentence.
   * @param data A Primitive or Structured value that contains additional information about the error. This may be omitted. The value of this member is defined by the Server (e.g. detailed error information, nested errors etc.).
   */
  public constructor(code: number, message: string, data?: object) {
    this.code = code;
    this.message = message;
    this.data = data;
  }
}

export interface IJsonRpcErrorResponse extends IJsonRpcResponse {
  error: IJsonRpcErrorObject;
}

export class JsonRpcErrorResponse implements IJsonRpcErrorResponse {
  error: IJsonRpcErrorObject;
  jsonrpc: string;
  id?: string | number | null;

  /**
   *
   * @param error This member is REQUIRED on error. This member MUST NOT exist if there was no error triggered during invocation. The value for this member MUST be an Object as defined in section 5.1.
   * @param jsonrpc A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
   * @param id This member is REQUIRED. It MUST be the same as the value of the id member in the Request Object. If there was an error in detecting the id in the Request object (e.g. Parse error/Invalid Request), it MUST be Null.
   */
  public constructor(error: IJsonRpcErrorObject, id?: string | number | null) {
    this.id = id;
    this.jsonrpc = '2.0';
    this.error = error;
  }
}

export class JsonRpcClient extends EventTarget {
  private _isConnected: boolean = false;
  private ws?: WebSocket;
  private readyStateOpen = 1;
  private url;

  public constructor(url: string | URL) {
    super();
    this.url = url;
  }

  public get isConnected(): boolean {
    return this._isConnected;
  }

  public set isConnected(newValue: boolean) {
    if (newValue != this._isConnected) {
      this.dispatchEvent(
        new CustomEvent<boolean>('isConnected', {
          detail: newValue
        })
      );
      this._isConnected = newValue;
    }
  }

  public async connect(): Promise<boolean> {
    this.ws = new WebSocket(this.url);
    this.isConnected = false;

    let result: Promise<boolean> = new Promise<boolean>((resolve, reject) => {
      this.ws!.onopen = (event: Event) => {
        console.log('connect ws.onopen', event);
        this.isConnected = true;
        resolve(true);
      };

      this.ws!.onclose = (event: CloseEvent) => {
        this.isConnected = false;
        console.log('connect ws.onclose', event);
        reject(event);
      };

      this.ws!.onerror = (event: Event) => {
        this.isConnected = false;
        console.log('connect ws.onerror', event);
        reject(event);
      };

      this.ws!.onmessage = (event: MessageEvent) => {
        const request: IJsonRpcRequest | IJsonRpcRequest[] = JSON.parse(event.data);

        // console.log('connect ws.onmessage: single', request);
        if (Array.isArray(request)) {
          // batch request - send notification for every notification inside
          console.log('connect ws.onmessage: received batchRequest', request);
          request.forEach((singleRequest) => {
            if (!singleRequest.id) {
              //console.log('connect ws.onmessage: notification', request);
              this.dispatchEvent(
                new CustomEvent<IJsonRpcRequest[]>('notification', {
                  detail: request
                })
              );
            }
          });
        } else {
          // single request
          if (!request.id) {
            //console.log('connect ws.onmessage: notification', request);
            this.dispatchEvent(
              new CustomEvent<IJsonRpcRequest>('notification', {
                detail: request
              })
            );
          }
        }
      };
    });

    return result;
  }

  public async disconnect(): Promise<boolean> {
    let result: Promise<boolean> = new Promise<boolean>((resolve, reject) => {
      if (this.isConnected) {
        this.ws!.close();
        resolve(true);
      } else {
        reject('WebSocket was not connected');
      }
    });

    this.ws = undefined;

    return result;
  }

  public async sendRequest(request: IJsonRpcRequest): Promise<IJsonRpcSuccessResponse | IJsonRpcErrorResponse> {
    let promise: Promise<IJsonRpcSuccessResponse | IJsonRpcErrorResponse>;

    if (!this.isConnected) {
      console.log('sendRequest ws not connected');
      promise = Promise.reject('Reject: ws not connected');
    } else if (this.ws!.readyState == this.readyStateOpen) {
      console.log('sendRequest ws.readyState', request);

      promise = new Promise<IJsonRpcSuccessResponse | IJsonRpcErrorResponse>((resolve, reject) => {
        let timeout = setTimeout(() => {
          this.ws!.removeEventListener('message', parser);

          console.log('sendRequest - timeout');
          reject('sendRequest - timeout');
        }, 30 * 1000);

        let parser = (event: MessageEvent) => {
          const response: IJsonRpcSuccessResponse | IJsonRpcErrorResponse = JSON.parse(event.data);

          if (request.id == response.id) {
            this.ws!.removeEventListener('message', parser);
            clearTimeout(timeout);

            console.log('parser response for id:', response.id, 'response:', response);

            resolve(response);
          } else {
            console.log('parser some message - request: ', request, ' response:', response);
          }
        };
        this.ws!.addEventListener('message', parser);
      });

      this.ws!.send(JSON.stringify(request));
    } else {
      console.log('ws not ready reject');
      promise = Promise.reject('ws not ready');
    }

    return promise;
  }

  public async sendBatchRequest(requests: IJsonRpcRequest[]): Promise<IJsonRpcSuccessResponse[] | IJsonRpcErrorResponse[]> {
    let promise: Promise<IJsonRpcSuccessResponse[] | IJsonRpcErrorResponse[]>;

    if (!this.isConnected) {
      console.log('sendBatchRequest ws not connected');
      promise = Promise.reject('Reject: ws not connected');
    } else if (this.ws!.readyState == this.readyStateOpen) {
      console.log('sendBatchRequest ws.readyState: open', requests);

      promise = new Promise<IJsonRpcSuccessResponse[] | IJsonRpcErrorResponse[]>((resolve, reject) => {
        let timeout = setTimeout(() => {
          this.ws!.removeEventListener('message', parser);

          console.log('sendBatchRequest - timeout');
          reject('sendBatchRequest - timeout');
        }, 30 * 1000);

        let parser = (event: MessageEvent) => {
          const responses: IJsonRpcSuccessResponse[] | IJsonRpcErrorResponse[] = JSON.parse(event.data);

          // TODO if request is a invalid json -> response is a single error json. code don't care about this right now
          if (Array.isArray(responses)) {
            console.log('BatchResponses', responses);

            // TODO result could have another order than requests.
            let responsesWithId: IJsonRpcResponse[] = new Array<IJsonRpcResponse>();

            for (let index = 0; index < responses.length; index++) {
              const element = responses[index];

              if (element.id) [responsesWithId.push(element)];
            }

            if (responsesWithId.length >= 1) {
              this.ws!.removeEventListener('message', parser);
              clearTimeout(timeout);

              console.log('parser response for id:', requests[0].id, 'responsesWithId:', responsesWithId);

              resolve(responses);
            } else {
              console.log('parser some message - requests:', requests, ' responses:', responses);
            }
          }
        };
        this.ws!.addEventListener('message', parser);
      });

      this.ws!.send(JSON.stringify(requests));
    } else {
      console.log('ws not ready reject');
      promise = Promise.reject('ws not ready');
    }

    return promise;
  }

  public static generateConnectionId(): string {
    return Math.floor(Math.random() * 1000000).toString();
  }
}
