import { JSX } from "preact";
import { computed, signal } from "@preact/signals";

import Label from "#components/form/Label.tsx";
import Password from "#islands/form/Password.tsx";
import Text from "#islands/form/Text.tsx";
import Button from "#components/form/Button.tsx";
import ErrorBox from "#islands/ErrorBox.tsx";
import { signin } from "#utils/opaque.ts";

export default function SignInForm() {
  const username = signal("");
  const password = signal("");
  const disabled = computed(() =>
    password.value === "" || username.value === ""
  );

  const errorMessage = signal<string | undefined>(undefined);
  const onSubmit = async (event: JSX.TargetedSubmitEvent<HTMLFormElement>) => {
    event.preventDefault();

    try {
      const form = new FormData(event.currentTarget);
      await signin({
        username: form.get("username") as string,
        password: form.get("password") as string,
      });
      errorMessage.value = undefined;
      window.location.href = "/";
    } catch (err) {
      errorMessage.value = (err instanceof Error)
        ? err.message
        : err.toString();
    }
  };

  return (
    <form
      class="flex flex-col gap-6 w-96 m-auto p-5 bg-gray-50 border border-gray-300 rounded shadow"
      onSubmit={onSubmit}
    >
      <div class="flex flex-row">
        <h1 class="mx-auto text-2xl font-bold text-gray-900">Sign in</h1>
      </div>
      <div class="flex flex-col gap-1">
        <Label for="username">Username or email address</Label>
        <Text
          id="username"
          name="username"
          value={username}
        />
      </div>
      <div class="flex flex-col gap-1">
        <Label for="password">Password</Label>
        <Password id="password" name="password" value={password} />
      </div>
      <div class="flex flex-row">
        <div class="mx-auto">
          <Button
            type="submit"
            disabled={disabled}
          >
            Sign in
          </Button>
        </div>
      </div>
      <ErrorBox message={errorMessage} />
    </form>
  );
}
