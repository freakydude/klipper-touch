import type { IJsonRpcErrorObject } from './IJsonRpcErrorObject';
import type { IJsonRpcResponse } from './IJsonRpcResponse';

export interface IJsonRpcErrorResponse extends IJsonRpcResponse {
  error: IJsonRpcErrorObject; // This member is REQUIRED on error. This member MUST NOT exist if there was no error triggered during invocation. The value for this member MUST be an Object as defined in IJsonRpcErrorObject.
}
