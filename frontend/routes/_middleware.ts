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
    ctx.url.pathname === "/signin" || ctx.destination !== "route";
  const isExcludedRoute = <T>(ctx: FreshContext<T>) =>
    excludedPath.includes(ctx.url.pathname) || ctx.destination !== "route";

  // check if the user is already signed in
  const cookies = getCookies(req.headers);
  const sessionId = cookies["FRESH_SESSION"];
  if (sessionId === undefined) {
    if (isExcludedRoute(ctx)) {
      return await ctx.next();
    }

    return new Response(null, {
      status: 307,
      headers: { "location": "/signin" },
    });
  }

  const response = await api.get<SessionRes>(`/session/${sessionId}`);
  if (!response.ok) {
    return new Response(null, {
      status: 307,
      headers: {
        "location": "/signin",
        "set-cookie": "FRESH_SESSION=; Path=/; Max-Age=0",
      },
    });
  }

  // logged in user that try to access to /sigin endpoint are automatically
  // redirected to the landing page
  if (isSigninRoute(ctx)) {
    return new Response(null, {
      status: 307,
      headers: { "location": "/" },
    });
  }

  ctx.state.session = response.data;
  return await ctx.next();
}
