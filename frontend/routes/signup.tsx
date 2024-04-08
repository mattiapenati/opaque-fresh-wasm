import { Head } from "$fresh/runtime.ts";
import { Handlers, PageProps } from "$fresh/server.ts";

import SignUpForm from "#islands/SignUpForm.tsx";

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
  GET(req, ctx) {
    const reqUrl = new URL(req.url);
    const code = reqUrl.searchParams.get("code");
    let username;
    try {
      if (code) {
        const encoded_invitation = code.split(".")[0];
        const invitation = JSON.parse(atob(encoded_invitation));
        username = invitation.username;
      }
    } catch (err) {
      console.error(err);
    }

    return ctx.render({
      code,
      username,
    });
  },
};
