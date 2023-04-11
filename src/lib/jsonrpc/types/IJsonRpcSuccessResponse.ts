import type { IJsonRpcResponse } from './IJsonRpcResponse';

export interface IJsonRpcSuccessResponse extends IJsonRpcResponse {
  result: object; // This member is REQUIRED on success. This member MUST NOT exist if there was an error invoking the method. The value of this member is determined by the method invoked on the Server.
}
