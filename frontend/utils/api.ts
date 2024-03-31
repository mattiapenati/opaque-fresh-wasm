// @deno-types="../wasm/fresh_auth_frontend.d.ts"
import {
  checkCredentialsStrength,
  OpaqueLogin,
  OpaqueRegistration,
} from "../wasm/fresh_auth_frontend.js";

interface Invitation {
  username?: string;
  expiration?: string;
}

const API_URL = "http://localhost:8080";

export const fetchInvitationUsername = async (
  code: string,
): Promise<string | undefined> => {
  const response = await fetch(`${API_URL}/api/signup/invitation/${code}`);
  if (!response.ok) {
    return;
  }
  const invitation = await response.json() as Invitation;
  return invitation.username;
};

export interface SignupArgs {
  code: string;
  username: string;
  password: string;
}

export const signup = async ({ code, username, password }: SignupArgs) => {
  checkCredentialsStrength(username, password);
  const opaqueRegistration = OpaqueRegistration.start(password);
  const { session, message: startMessage } = await signupStart({
    code,
    username,
    message: opaqueRegistration.message,
  });
  const { message: finishMessage } = opaqueRegistration.finish(
    password,
    startMessage,
  );
  await signupFinish({ session, message: finishMessage });
};

interface SignupStartReq {
  code: string;
  username: string;
  message: string;
}

interface SignupStartRes {
  session: string;
  message: string;
}

const signupStart = async (req: SignupStartReq) => {
  const response = await fetch("/api/signup/start", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(req),
  });
  if (response.status === 401) {
    throw new Error("Invalid credentials");
  }
  if (!response.ok) {
    throw new Error("Sign up server is not available");
  }

  return await response.json() as SignupStartRes;
};

interface SignupFinishReq {
  session: string;
  message: string;
}

const signupFinish = async (req: SignupFinishReq) => {
  const response = await fetch("/api/signup/finish", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(req),
  });

  if (response.status === 401) {
    throw new Error("Invalid credentials");
  }
  if (!response.ok) {
    throw new Error("Signup server is not available");
  }
};

export interface SigninArgs {
  username: string;
  password: string;
}

export const signin = async ({ username, password }: SigninArgs) => {
  const opaqueLogin = OpaqueLogin.start(password);
  const { session, message: startMessage } = await signinStart({
    username,
    message: opaqueLogin.message,
  });
  const { message: finishMessage } = opaqueLogin.finish(password, startMessage);
  await signinFinish({ session, message: finishMessage });
};

interface SigninStartReq {
  username: string;
  message: string;
}

interface SigninStartRes {
  session: string;
  message: string;
}

const signinStart = async (req: SigninStartReq) => {
  const response = await fetch("/api/signin/start", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(req),
  });
  if (response.status === 401) {
    throw new Error("Invalid credentials");
  }
  if (!response.ok) {
    throw new Error("Sign in server is not available");
  }

  return await response.json() as SigninStartRes;
};

interface SigninFinishReq {
  session: string;
  message: string;
}

const signinFinish = async (req: SigninFinishReq) => {
  const response = await fetch("/api/signin/finish", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(req),
  });
  if (response.status === 401) {
    throw new Error("Invalid credentials");
  }
  if (!response.ok) {
    throw new Error("Sign in server is not available");
  }
};
