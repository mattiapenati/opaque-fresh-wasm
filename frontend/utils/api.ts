import { IS_BROWSER } from "$fresh/runtime.ts";

export interface ApiResponseOk<T> {
  ok: true;
  data: T;
}

export interface ApiResponseErr {
  ok: false;
  status: number;
}

export type ApiResponse<T> = ApiResponseOk<T> | ApiResponseErr;

export interface ApiOptions {
  token?: string;
}

interface ApiRequest<T> {
  method: string;
  path: string;
  body?: T;
}

export class Api {
  readonly #url: string;
  readonly #token?: string;

  constructor(url: string, { token }: ApiOptions = {}) {
    this.#url = new URL("/api", url).href;
    this.#token = token;
  }

  get<Res>(path: string): Promise<ApiResponse<Res>> {
    return this.request({
      method: "GET",
      path,
    });
  }

  post<Res = unknown, Req = unknown>(
    path: string,
    body: Req,
  ): Promise<ApiResponse<Res>> {
    return this.request({
      method: "POST",
      path,
      body,
    });
  }

  private async request<Res = unknown, Req = unknown>(
    request: ApiRequest<Req>,
  ): Promise<ApiResponse<Res>> {
    const url = new URL(`${this.#url}${request.path}`);
    const headers = new Headers();
    if (this.#token) {
      headers.set("authorization", `Bearer ${this.#token}`);
    }
    if (request.body) {
      headers.set("content-type", "application/json");
    }
    const body = request.body ? JSON.stringify(request.body) : undefined;

    const response = await fetch(url.href, {
      method: request.method,
      headers,
      body,
    });

    if (response.ok) {
      const data = await response.json() as Res;
      return { ok: true, data };
    } else {
      return { ok: false, status: response.status };
    }
  }
}

let api: Api;
if (IS_BROWSER) {
  const url = location.href;
  api = new Api(url);
} else {
  const default_url = "http://localhost:8080";
  const url = Deno.env.get("API_URL") ?? default_url;
  const token = Deno.env.get("API_TOKEN");
  api = new Api(url, { token });
}
export { api };

/** Invitation detail */
export interface Invitation {
  username: string;
  expiration: string;
}

/** Sign up start step request */
export interface SignupStartReq {
  code: string;
  username: string;
  message: string;
}

/** Sign up start step response */
export interface SignupStartRes {
  session: string;
  message: string;
}

/** Sign up finish step request */
export interface SignupFinishReq {
  session: string;
  message: string;
}

/** Sign in start step request */
export interface SigninStartReq {
  username: string;
  message: string;
}

/** Sign in start step response */
export interface SigninStartRes {
  session: string;
  message: string;
}

/** Sign in finish step request */
export interface SigninFinishReq {
  session: string;
  message: string;
}

/** Session information */
export interface SessionRes {
  username: string;
}
