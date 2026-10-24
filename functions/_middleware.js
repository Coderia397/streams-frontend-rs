// Keeps the two hosts of one deployment apart (note 80, 2026-10-18): the Help Centre lives on help.streams.watch, the app and the policies on streams.watch.
//   streams.watch/help...      -> 301 help.streams.watch/help...
//   help.streams.watch/<app>   -> 301 streams.watch/<app>   (files and the root are left alone; the root shows the Help Centre in the app)
// and each host gets its own sitemap and the sitemap line of its robots file. Only the paths in public/_routes.json reach this file.
//
// It also gives every page its own head (note 117): the page is built in the browser, but link previews (WhatsApp, Messages, X, Snapchat,
// Discord, Slack...) and search engines read the head of the HTML as it arrives. A title's page (/title/tv-1399) gets "Watch <name> | Streams",
// its synopsis, its backdrop as the preview picture, its own canonical address and its schema.org data; the other public pages get their own
// name and canonical address (all of them pointed at the home page before, which tells search engines they are copies of it).
const MAIN = "streams.watch";
const HELP = "help.streams.watch";
const FILE = /\.[a-z0-9]{2,5}$/i;
const BRAND = "Streams";
const TITLE_PATH = /^\/title\/(tv|movie)-([1-9]\d{0,9})$/;
// the site's TMDB key, the same one every visitor's browser already uses (src/config.rs); STREAMS_TMDB_KEY in the Pages settings replaces it
const PUBLIC_TMDB_KEY = "fc5fec3b73d8605daaeb1eb3b91157eb";
const TMDB_IMAGES = "https://image.tmdb.org/t/p";

// the public pages that are not titles: their name and what a preview says about them
const PAGES = {
  "/login": ["Sign In", "Sign in to Streams and pick up where you left off."],
  "/signup": ["Sign Up", "Make a free Streams account: profiles, a Kids profile and rows made for your taste."],
  "/forgot": ["Reset Your Password", "Get a link to set a new password for your Streams account."],
  "/help": ["Help Centre", "Answers about your account, watching on your devices, profiles and more."],
  "/terms": ["Terms of Service", "The terms for using Streams."],
  "/privacy": ["Privacy Policy", "What Streams collects, why, and the choices you have."],
  "/cookies": ["Cookie Policy", "The cookies and storage Streams uses, and how to choose."],
  "/dmca": ["DMCA Safe Harbor Notice", "How to send a copyright notice to Streams."],
  "/disclaimer": ["Disclaimer", "What Streams is and what it is not."],
  "/credits": ["Licenses and credits", "The data, software and art Streams is made with."],
  "/clips": ["Clips", "Trailers to watch one after another."],
  "/latest": ["New & Popular", "What is new and what everyone is watching."],
  "/search": ["Search", "Find films and series on Streams."],
};

export async function onRequest({ request, next, env }) {
  const url = new URL(request.url);
  const host = url.hostname.toLowerCase();
  const ours = host === MAIN || host === `www.${MAIN}` || host === HELP;
  const p = url.pathname;
  if (ours) {
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
      // the help host lists its own pages only (the titles' sitemap is the main host's)
      const text = (await res.text()).replace(`https://${MAIN}/sitemap.xml`, `https://${HELP}/sitemap.xml`).replace(/^Sitemap: \S*sitemap-titles\.xml\n?/m, "");
      return new Response(text, { status: res.status, headers: res.headers });
    }
  }
  const res = await next();
  if (request.method !== "GET" || FILE.test(p) || !(res.headers.get("content-type") || "").includes("text/html")) return res;
  // the canonical host: the help pages on help.streams.watch, everything else on streams.watch (previews keep their own host)
  const origin = ours ? `https://${p === "/help" || p.startsWith("/help/") ? HELP : MAIN}` : url.origin;
  const title = TITLE_PATH.exec(p);
  if (title) {
    const head = await titleHead(title[1], title[2], `${origin}${p}`, env).catch(() => null);
    if (head) return withHead(res, head);
  }
  const page = PAGES[p] || (p.startsWith("/help/") ? PAGES["/help"] : null);
  return withHead(res, {
    title: page ? `${page[0]} | ${BRAND}` : null,
    description: page ? page[1] : null,
    url: `${origin}${p === "/" ? "/" : p}`,
  });
}

/** The head of a title's page, from TMDB (kept a day at the edge). */
async function titleHead(kind, id, pageUrl, env) {
  const key = (env && env.STREAMS_TMDB_KEY) || PUBLIC_TMDB_KEY;
  const res = await fetch(`https://api.themoviedb.org/3/${kind}/${id}?api_key=${key}&language=en-US`, { cf: { cacheTtl: 86400, cacheEverything: true } });
  if (!res.ok) return null;
  const d = await res.json();
  const name = d.title || d.name;
  if (!name) return null;
  const date = d.release_date || d.first_air_date || "";
  const year = date.slice(0, 4);
  const picture = d.backdrop_path ? `${TMDB_IMAGES}/w1280${d.backdrop_path}` : d.poster_path ? `${TMDB_IMAGES}/w780${d.poster_path}` : null;
  const description = shorten(d.overview || "", 200) || `Watch ${name} on ${BRAND}.`;
  const data = {
    "@context": "https://schema.org",
    "@type": kind === "tv" ? "TVSeries" : "Movie",
    name,
    url: pageUrl,
    description: d.overview || undefined,
    image: d.poster_path ? `${TMDB_IMAGES}/w780${d.poster_path}` : picture || undefined,
    genre: (d.genres || []).map((g) => g.name),
    datePublished: date || undefined,
    numberOfSeasons: kind === "tv" ? d.number_of_seasons : undefined,
    duration: kind === "movie" && d.runtime ? `PT${d.runtime}M` : undefined,
  };
  return {
    title: `Watch ${name}${year ? ` (${year})` : ""} | ${BRAND}`,
    description,
    url: pageUrl,
    image: picture,
    imageAlt: name,
    imageSize: d.backdrop_path ? [1280, 720] : null,
    type: kind === "tv" ? "video.tv_show" : "video.movie",
    data,
  };
}

/** `text` cut at a word before `max` characters, with an ellipsis when it was cut. */
function shorten(text, max) {
  const t = text.replace(/\s+/g, " ").trim();
  if (t.length <= max) return t;
  const cut = t.slice(0, max - 1);
  return `${cut.slice(0, Math.max(cut.lastIndexOf(" "), max / 2)).replace(/[\s,;:.]+$/, "")}…`;
}

/** The response with its head's title, description, canonical address, preview tags and schema.org data replaced where given. */
function withHead(res, h) {
  const set = (attr, value) => ({ element(e) { if (value != null) e.setAttribute(attr, value); } });
  let rw = new HTMLRewriter()
    .on("title", { element(e) { if (h.title) e.setInnerContent(h.title); } })
    .on('meta[name="description"]', set("content", h.description))
    .on('meta[property="og:description"]', set("content", h.description))
    .on('meta[name="twitter:description"]', set("content", h.description))
    .on('meta[property="og:title"]', set("content", h.title))
    .on('meta[name="twitter:title"]', set("content", h.title))
    .on('meta[property="og:url"]', set("content", h.url))
    .on('link[rel="canonical"]', set("href", h.url))
    .on('meta[property="og:image"]', set("content", h.image))
    .on('meta[property="og:type"]', set("content", h.type));
  const extra = [];
  if (h.image) extra.push(`<meta name="twitter:image" content="${attr(h.image)}" />`, `<meta property="og:image:alt" content="${attr(h.imageAlt || "")}" />`);
  if (h.imageSize) extra.push(`<meta property="og:image:width" content="${h.imageSize[0]}" />`, `<meta property="og:image:height" content="${h.imageSize[1]}" />`);
  // `<` written as \u003c, so a synopsis cannot close the script
  if (h.data) extra.push(`<script type="application/ld+json">${JSON.stringify(h.data).replace(/</g, "\\u003c")}</script>`);
  if (extra.length) rw = rw.on("head", { element(e) { e.append(extra.join(""), { html: true }); } });
  const out = rw.transform(res);
  // the body's length changed with the head
  const headers = new Headers(out.headers);
  headers.delete("content-length");
  return new Response(out.body, { status: out.status, headers });
}

function attr(s) {
  return String(s).replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;");
}
