import type { IJsonRpcErrorObject } from './IJsonRpcErrorObject';

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
  public constructor({ code, message, data }: IJsonRpcErrorObject) {
    this.code = code;
    this.message = message;
    this.data = data;
  }
}
