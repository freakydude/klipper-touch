import type { IJsonRpcSuccessResponse } from './IJsonRpcSuccessResponse';

export class JsonRpcSuccessResponse implements IJsonRpcSuccessResponse {
  result: object;
  jsonrpc: string;
  id: string | number | null;

  /**
   *
   * @param jsonrpc A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
   * @param id This member is REQUIRED. It MUST be the same as the value of the id member in the Request Object. If there was an error in detecting the id in the Request object (e.g. Parse error/Invalid Request), it MUST be Null.
   * @param result This member is REQUIRED on success. This member MUST NOT exist if there was an error invoking the method. The value of this member is determined by the method invoked on the Server.
   */
  constructor({ jsonrpc = '2.0', id, result }: IJsonRpcSuccessResponse) {
    this.jsonrpc = jsonrpc;
    this.id = id;
    this.result = result;
  }
}
