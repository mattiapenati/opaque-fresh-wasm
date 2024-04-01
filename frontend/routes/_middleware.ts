import { FreshContext } from "$fresh/server.ts";
import { getCookies } from "$std/http/cookie.ts";
import { api } from "#utils/api.ts";
import { SessionRes } from "#utils/api.ts";

interface State {
  session?: Session;
}

interface Session {
  username: string;
}

export async function handler(
  req: Request,
  ctx: FreshContext<State>,
) {
  // signin and signup page are always available
  const excludedPath = ["/signin", "/signup"];
  const isSigninRoute = <T>(ctx: FreshContext<T>) =>
    ctx.url.pathname === "/signin";
  const isExcludedRoute = <T>(ctx: FreshContext<T>) =>
    excludedPath.includes(ctx.url.pathname) || ctx.destination !== "route";

  // extract cookie, the signature is removed
  const cookies = getCookies(req.headers);
  const sessionId = cookies["SESSIONID"];

  // session cookie is missing, if the page is an excluded route the page is
  // loaded otherwise redirected to login page
  if (!sessionId) {
    if (isExcludedRoute(ctx)) {
      return await ctx.next();
    } else {
      return new Response(null, {
        status: 307,
        headers: { "location": "/signin" },
      });
    }
  }

  const response = await api.get<SessionRes>(`/session/${sessionId}`);
  // the session is not valid, remove the session cookie and reload same page
  if (!response.ok) {
    return new Response(null, {
      status: 307,
      headers: {
        "location": ctx.url.pathname,
        "set-cookie": "SESSIONID=; Path=/; Max-Age=0",
      },
    });
  }

  // logged in user that try to access to /signin endpoint are automatically
  // redirected to the landing page
  if (isSigninRoute(ctx)) {
    return new Response(null, {
      status: 307,
      headers: { "location": "/" },
    });
  }

  // set the session in the context state
  ctx.state.session = response.data;
  return await ctx.next();
}
