// Keeps the two hosts of one deployment apart (note 80, 2026-10-18): the Help Centre lives on help.streams.watch, the app and the policies on streams.watch.
//   streams.watch/help...      -> 301 help.streams.watch/help...
//   help.streams.watch/<app>   -> 301 streams.watch/<app>   (files and the root are left alone; the root shows the Help Centre in the app)
// and each host gets its own sitemap and the sitemap line of its robots file. Only the paths in public/_routes.json reach this file.
const MAIN = "streams.watch";
const HELP = "help.streams.watch";
const FILE = /\.[a-z0-9]{2,5}$/i;

export async function onRequest({ request, next }) {
  const url = new URL(request.url);
  const host = url.hostname.toLowerCase();
  if (host !== MAIN && host !== `www.${MAIN}` && host !== HELP) return next();    // pages.dev previews and anything else pass through
  const p = url.pathname;
  const onHelp = host === HELP;
  const helpPath = p === "/help" || p.startsWith("/help/");
  if (!onHelp && helpPath) return Response.redirect(`https://${HELP}${p}${url.search}`, 301);
  if (onHelp && !helpPath && p !== "/" && !FILE.test(p)) return Response.redirect(`https://${MAIN}${p}${url.search}`, 301);

  if (p === "/sitemap.xml") {
    const res = await next();
    const text = await res.text();
    const lines = text.split("\n").filter((l) => !l.includes("<loc>") || (l.includes(`${MAIN}/help`) === onHelp));
    const body = lines.join("\n").replaceAll(onHelp ? `https://${MAIN}/help` : "\u0000", `https://${HELP}/help`);
    return new Response(body, { status: res.status, headers: res.headers });
  }
  if (p === "/robots.txt" && onHelp) {
    const res = await next();
    const text = (await res.text()).replace(`https://${MAIN}/sitemap.xml`, `https://${HELP}/sitemap.xml`);
    return new Response(text, { status: res.status, headers: res.headers });
  }
  return next();
}
