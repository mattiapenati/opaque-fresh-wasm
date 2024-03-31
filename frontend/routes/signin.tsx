import { Head } from "$fresh/runtime.ts";
import SignInForm from "#islands/SignInForm.tsx";

export default function Signin() {
  return (
    <>
      <Head>
        <title>Fresh Auth | Signin</title>
      </Head>
      <div class="flex h-screen">
        <SignInForm />
      </div>
    </>
  );
}
