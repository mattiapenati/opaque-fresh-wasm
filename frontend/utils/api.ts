// @deno-types="../wasm/fresh_auth_frontend.d.ts"
import {
  checkCredentialsStrength,
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

export interface SignupProps {
  code: string;
  username: string;
  password: string;
}

export const signup = async ({ code, username, password }: SignupProps) => {
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
    throw new Error("Signup server is not available");
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
