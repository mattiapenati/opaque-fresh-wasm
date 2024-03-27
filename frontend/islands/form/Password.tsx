import { JSX } from "preact";
import { useState } from "preact/hooks";

import RiEyeLine from "#components/icons/RiEyeLine.tsx";
import RiEyeOffLine from "#components/icons/RiEyeOffLine.tsx";
import { isSignalLike } from "#utils/preact.ts";

interface Props extends JSX.HTMLAttributes<HTMLInputElement> {
  id: string;
}

export default function Password(props: Props) {
  const [showPassword, setShowPassword] = useState(false);

  return (
    <div class="flex flex-row items-center px-3 py-2 rounded bg-white ring-1 ring-inset ring-gray-300 focus-within:ring-2 focus-within:ring-inset focus-within:ring-primary-600">
      <input
        {...props}
        type={showPassword ? "text" : "password"}
        value={props.value}
        onInput={(e) => {
          if (isSignalLike(props.value)) {
            props.value.value = e.currentTarget.value;
          }
        }}
        class="block flex-1 p-0 m-0 border-0 bg-transparent text-gray-900 focus:ring-0"
      />
      <label for={`show-${props.id}`} class="grid w-5 h-5">
        <span class="sr-only">Show password</span>
        <input
          type="checkbox"
          id={`show-${props.id}`}
          checked={showPassword}
          onChange={() => setShowPassword(!showPassword)}
          class="appearance-none cursor-pointer opacity-0 row-start-1 col-start-1 w-full h-full"
        />
        <div class="flex overflow-hidden row-start-1 col-start-1">
          <div
            data-show={showPassword}
            class="data-[show=true]:hidden data-[show=false]:block"
          >
            <RiEyeLine class="w-full h-full fill-gray-900" />
          </div>
          <div
            data-hide={!showPassword}
            class="data-[hide=true]:hidden data-[hide=false]:block"
          >
            <RiEyeOffLine class="w-full h-full fill-gray-900" />
          </div>
        </div>
      </label>
    </div>
  );
}
