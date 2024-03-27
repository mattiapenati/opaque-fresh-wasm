import { SignalLike } from "$fresh/src/types.ts";

export function isSignalLike(value : string
  | string[]
  | number
  | undefined
  | SignalLike<string | string[] | number | undefined>): value is SignalLike<string | string[] | number | undefined> {
    return value instanceof Object && 'value' in value;
  }