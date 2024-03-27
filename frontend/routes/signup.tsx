import { Head } from "$fresh/runtime.ts";
import { Handlers, PageProps } from "$fresh/server.ts";

import SignUpForm from "#islands/SignUpForm.tsx";
import { fetchInvitationUsername } from "#utils/api.ts";

interface Data {
  code?: string;
  username?: string;
}

export default function SignUp({ data }: PageProps<Data>) {
  return (
    <>
      <Head>
        <title>Fresh Auth | Sign up</title>
      </Head>
      <div class="flex h-screen">
        <SignUpForm {...data} />
      </div>
    </>
  );
}

export const handler: Handlers = {
  async GET(req, ctx) {
    const reqUrl = new URL(req.url);
    const code = reqUrl.searchParams.get("code");
    const username = code && await fetchInvitationUsername(code);

    return ctx.render({
      code,
      username,
    });
  },
};
