import { JSX } from "preact";

import RiLockLine from "#components/icons/RiLockLine.tsx";
import { isSignalLike } from "#utils/preact.ts";

interface Props extends JSX.HTMLAttributes<HTMLInputElement> {
  id: string;
}

export default function Text(props: Props) {
  return (
    <div class="flex flex-row items-center px-3 py-2 rounded bg-white ring-1 ring-inset ring-gray-300 focus-within:ring-2 focus-within:ring-inset focus-within:ring-primary-600">
      <input
        {...props}
        type="text"
        value={props.value}
        onInput={(e) => {
          if (isSignalLike(props.value)) {
            props.value.value = e.currentTarget.value;
          }
        }}
        class="block flex-1 p-0 m-0 border-0 bg-transparent text-gray-900 focus:ring-0"
      />
      <div
        data-readonly={props.readOnly ?? false}
        class="h-5 w-5 data-[readonly=true]:block data-[readonly=false]:hidden"
      >
        <RiLockLine class="h-full w-full fill-gray-900" />
      </div>
    </div>
  );
}
