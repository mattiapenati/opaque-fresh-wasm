// DO NOT EDIT. This file is generated by Fresh.
// This file SHOULD be checked into source version control.
// This file is automatically updated during development when running `dev.ts`.

import * as $_404 from "./routes/_404.tsx";
import * as $_app from "./routes/_app.tsx";
import * as $signin from "./routes/signin.tsx";
import * as $signup from "./routes/signup.tsx";
import * as $ErrorBox from "./islands/ErrorBox.tsx";
import * as $SignInForm from "./islands/SignInForm.tsx";
import * as $SignUpForm from "./islands/SignUpForm.tsx";
import * as $form_Password from "./islands/form/Password.tsx";
import * as $form_Text from "./islands/form/Text.tsx";
import { type Manifest } from "$fresh/server.ts";

const manifest = {
  routes: {
    "./routes/_404.tsx": $_404,
    "./routes/_app.tsx": $_app,
    "./routes/signin.tsx": $signin,
    "./routes/signup.tsx": $signup,
  },
  islands: {
    "./islands/ErrorBox.tsx": $ErrorBox,
    "./islands/SignInForm.tsx": $SignInForm,
    "./islands/SignUpForm.tsx": $SignUpForm,
    "./islands/form/Password.tsx": $form_Password,
    "./islands/form/Text.tsx": $form_Text,
  },
  baseUrl: import.meta.url,
} satisfies Manifest;

export default manifest;
