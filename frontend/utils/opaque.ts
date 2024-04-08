// @deno-types="../wasm/fresh_auth_frontend.d.ts"
import {
  checkCredentialsStrength,
  OpaqueLogin,
  OpaqueRegistration,
} from "../wasm/fresh_auth_frontend.js";
import {
  api,
  SigninFinishReq,
  SigninStartReq,
  SigninStartRes,
} from "#utils/api.ts";

/** Sign up arguments */
export interface SignupArgs {
  code: string;
  username: string;
  password: string;
}

/** Send the requests for sign up process */
export const signup = async ({ code, username, password }: SignupArgs) => {
  checkCredentialsStrength(username, password);
  const opaqueRegistration = OpaqueRegistration.start(password);
  const { session, message: startMessage } = await signupStart({
    code,
    message: opaqueRegistration.message,
  });
  const { message: finishMessage } = opaqueRegistration.finish(
    password,
    startMessage,
  );
  await signupFinish({ session, message: finishMessage });
};

/** Sign up start step request */
interface SignupStartReq {
  code: string;
  message: string;
}

/** Sign up start step response */
interface SignupStartRes {
  session: string;
  message: string;
}

/** Sig up start step */
const signupStart = async (req: SignupStartReq) => {
  const response = await fetch("/signup", {
    method: "POST",
    body: JSON.stringify(Object.assign({ step: "start" }, req)),
    headers: { "content-type": "application/json" },
  });
  if (response.ok) {
    return await response.json() as SignupStartRes;
  }
  if (response.status === 401) {
    throw new Error("Invalid credentials");
  }
  throw new Error("Api server is not available");
};

/** Sign up finish step request */
interface SignupFinishReq {
  session: string;
  message: string;
}

/** Sign up finish step */
const signupFinish = async (req: SignupFinishReq) => {
  const response = await fetch("/signup", {
    method: "POST",
    body: JSON.stringify(Object.assign({ step: "finish" }, req)),
    headers: { "content-type": "application/json" },
  });
  if (response.ok) {
    await response.blob();
    return;
  }
  if (response.status === 401) {
    throw new Error("Invalid credentials");
  }
  throw new Error("Api server is not available");
};

/** Sign in arguments */
export interface SigninArgs {
  username: string;
  password: string;
}

/** Send the requests for sign in process */
export const signin = async ({ username, password }: SigninArgs) => {
  const opaqueLogin = OpaqueLogin.start(password);
  const { session, message: startMessage } = await signinStart({
    username,
    message: opaqueLogin.message,
  });
  const { message: finishMessage } = opaqueLogin.finish(password, startMessage);
  await signinFinish({ session, message: finishMessage });
};

const signinStart = async (req: SigninStartReq) => {
  const response = await api.post<SigninStartRes>("/signin/start", req);
  if (response.ok) {
    return response.data;
  }

  if (response.status === 401) {
    throw new Error("Invalid username or password");
  }
  throw new Error("Api server is not available");
};

const signinFinish = async (req: SigninFinishReq) => {
  const response = await api.post("/signin/finish", req);
  if (response.ok) {
    return;
  }

  if (response.status === 401) {
    throw new Error("Invalid username or password");
  }
  throw new Error("Api server is not available");
};
