import { JSX } from "preact";
import { IS_BROWSER } from "$fresh/runtime.ts";

export default function Button(props: JSX.HTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      {...props}
      disabled={!IS_BROWSER || props.disabled}
      class="rounded font-medium text-white text-sm text-center px-3 py-2 bg-primary-700 hover:bg-primary-800 ring-offset-1 focus:ring-2 focus:outline-none focus:ring-primary-300 disabled:bg-gray-600"
    />
  );
}
