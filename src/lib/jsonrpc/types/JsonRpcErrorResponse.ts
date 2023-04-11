import type { IJsonRpcErrorObject } from './IJsonRpcErrorObject';
import type { IJsonRpcErrorResponse } from './IJsonRpcErrorResponse';

export class JsonRpcErrorResponse implements IJsonRpcErrorResponse {
  error: IJsonRpcErrorObject;
  jsonrpc: string;
  id: string | number | null;

  /**
   *
   * @param error This member is REQUIRED on error. This member MUST NOT exist if there was no error triggered during invocation. The value for this member MUST be an Object as defined in section 5.1.
   * @param jsonrpc A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
   * @param id This member is REQUIRED. It MUST be the same as the value of the id member in the Request Object. If there was an error in detecting the id in the Request object (e.g. Parse error/Invalid Request), it MUST be Null.
   */
  public constructor({ jsonrpc = '2.0', id, error }: IJsonRpcErrorResponse) {
    this.id = id;
    this.jsonrpc = jsonrpc;
    this.error = error;
  }
}
