import { SignalLike } from "$fresh/src/types.ts";
import { JSX } from "preact";

import { isSignalLike } from "#utils/preact.ts";

interface Props extends JSX.HTMLAttributes<HTMLDivElement> {
  message?: string | SignalLike<string | undefined>;
}

export default function ErrorBox(props: Props) {
  const { message, ...divProps } = props;
  const messageValue = (isSignalLike(message)) ? message.value : message;

  return (messageValue === undefined) ? <></> : (
    <div
      {...divProps}
      class="text-xs font-bold p-2 rounded border-2 border-red-500 bg-red-200"
    >
      {messageValue}
    </div>
  );
}
