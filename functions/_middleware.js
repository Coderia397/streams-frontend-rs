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
  "/help/legal/terms": ["Terms of Service", "The terms for using Streams."],
  "/help/legal/privacy": ["Privacy Policy", "What Streams collects, why, and the choices you have."],
  "/help/legal/cookies": ["Cookie Policy", "The cookies and storage Streams uses, and how to choose."],
  "/help/legal/dmca": ["DMCA Safe Harbor Notice", "How to send a copyright notice to Streams."],
  "/help/legal/disclaimer": ["Disclaimer", "What Streams is and what it is not."],
  "/help/legal/credits": ["Licenses and credits", "The data, software and art Streams is made with."],
  "/clips": ["Clips", "Trailers to watch one after another."],
  "/latest": ["New & Popular", "What is new and what everyone is watching."],
  "/search": ["Search", "Find films and series on Streams."],
};

// the policies' earlier addresses and the ones people type, answered with a 301 to the policy in the Help Centre (src/config.rs, routes::MOVED;
// public/_routes.json must list each one, or this function never sees it)
const MOVED = {
  "/privacy": "/help/legal/privacy", "/privacy-policy": "/help/legal/privacy", "/privacy-statement": "/help/legal/privacy",
  "/terms": "/help/legal/terms", "/terms-of-service": "/help/legal/terms", "/terms-of-use": "/help/legal/terms", "/tos": "/help/legal/terms",
  "/cookies": "/help/legal/cookies", "/cookie-policy": "/help/legal/cookies", "/dmca": "/help/legal/dmca", "/disclaimer": "/help/legal/disclaimer",
  "/credits": "/help/legal/credits", "/licenses": "/help/legal/credits", "/legal": "/help/legal/privacy", "/help/legal": "/help/legal/privacy",
};

const APP_WASM = /^\/streams-frontend-rs-[0-9a-f]+_bg\.wasm$/;
// the home page's title and summary in search results and link previews (owner, 2026-10-24): the brand alone matched nothing people type
const HOME_TITLE = `${BRAND}: Free Films and Series, Picked for You`;
const HOME_DESCRIPTION = `Find your next film or series on ${BRAND}. Rows picked for your taste, trailers, a Kids profile, and no subscription or card.`;

export async function onRequest({ request, next, env }) {
  const url = new URL(request.url);
  // The app's wasm, the largest file of every first visit: the copy compressed once at brotli's highest level by the build
  // (scripts/precompress.mjs), about a quarter smaller than Cloudflare's on-the-fly compression (note 119). Without that copy, or for
  // a browser without brotli, the file is served as before.
  if (APP_WASM.test(url.pathname) && request.method === "GET" && /\bbr\b/.test(request.headers.get("accept-encoding") || "") && env && env.ASSETS) {
    const br = await env.ASSETS.fetch(new Request(new URL(`${url.pathname}.br`, url.origin))).catch(() => null);
    if (br && br.ok) {
      return new Response(br.body, {
        headers: {
          "content-type": "application/wasm",
          "content-encoding": "br",
          "cache-control": "public, max-age=31536000, immutable",
          vary: "accept-encoding",
        },
        encodeBody: "manual",
      });
    }
    return next();
  }
  const host = url.hostname.toLowerCase();
  const ours = host === MAIN || host === `www.${MAIN}` || host === HELP;
  const p = url.pathname;
  if (ours) {
    const onHelp = host === HELP;
    const moved = MOVED[p.replace(/\/+$/, "").toLowerCase()];
    if (moved) return Response.redirect(`https://${HELP}${moved}${url.search}`, 301);
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
    // a title TMDB does not have goes to the root, as every address the site does not have (src/lib.rs): no page of its own
    if (head && head.missing) return Response.redirect(`${url.origin}/`, 302);
    if (head) return withHead(res, head, head.firstView);
  }
  // the Help Centre's home and articles: every one was the same empty page before the app ran, so Google kept one of them and
  // called the rest duplicates ("Google chose different canonical than user", Search Console, 2026-10-24)
  const article = /^\/help\/article\/([a-z0-9-]+)$/.exec(p);
  if ((p === "/help" || article) && env && env.ASSETS) {
    const help = await helpFirstView(env, url.origin, article ? article[1] : null).catch(() => null);
    if (help) return withHead(res, { title: article ? `${help.title} | ${BRAND} Help Centre` : `${help.title} | ${BRAND}`, description: help.description, url: `${origin}${p}` }, help.html);
  }
  // the policies: before this, a reader without the app found only their titles, so who runs the site and its terms were unreadable
  if (LEGAL[p] && env && env.ASSETS) {
    const legal = await legalFirstView(env, url.origin, p).catch(() => null);
    if (legal) return withHead(res, { title: `${PAGES[p][0]} | ${BRAND} Help Centre`, description: PAGES[p][1], url: `${origin}${p}` }, legal);
  }
  if (p === "/" && !(ours && host === HELP) && env && env.ASSETS) {
    const first = await landingFirstView(env, url.origin).catch(() => null);
    return withHead(res, { title: HOME_TITLE, description: HOME_DESCRIPTION, url: `${origin}/` }, first);
  }
  const page = PAGES[p] || (p.startsWith("/help/") ? PAGES["/help"] : null);
  return withHead(res, {
    title: page ? `${page[0]} | ${BRAND}` : null,
    description: page ? page[1] : null,
    url: `${origin}${p === "/" ? "/" : p}`,
  });
}

/** The head of a title's page and its first view, from TMDB (kept a day at the edge). */
async function titleHead(kind, id, pageUrl, env) {
  const key = (env && env.STREAMS_TMDB_KEY) || PUBLIC_TMDB_KEY;
  const res = await fetch(`https://api.themoviedb.org/3/${kind}/${id}?api_key=${key}&language=en-US&append_to_response=credits,recommendations,external_ids`, { cf: { cacheTtl: 86400, cacheEverything: true } });
  if (res.status === 404) return { missing: true };
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
    // the people and the same work elsewhere, so search engines tie the page to the right film or series (not to another of the name)
    actor: person(((d.credits && d.credits.cast) || []).slice(0, 5).map((c) => c.name)),
    director: kind === "movie" ? person(((d.credits && d.credits.crew) || []).filter((c) => c.job === "Director").map((c) => c.name)) : undefined,
    creator: kind === "tv" ? person((d.created_by || []).map((c) => c.name)) : undefined,
    sameAs: [
      d.external_ids && d.external_ids.imdb_id ? `https://www.imdb.com/title/${d.external_ids.imdb_id}/` : null,
      `https://www.themoviedb.org/${kind}/${id}`,
    ].filter(Boolean),
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
    firstView: titleFirstView(kind, id, d, name, year),
  };
}

/** The title's page as plain HTML, shown while the app downloads (a slow phone waits seconds for it) and read by search engines without
 *  running any script: the backdrop, name, year, genres, synopsis, cast, the sign-in link the app's page offers, and links to titles like
 *  it. The app removes it once it has started (src/main.rs). Colours are the site's own tokens (style/output.css is already loaded). */
function titleFirstView(kind, id, d, name, year) {
  const genres = (d.genres || []).map((g) => g.name).slice(0, 3);
  const length = kind === "tv"
    ? (d.number_of_seasons ? `${d.number_of_seasons} season${d.number_of_seasons > 1 ? "s" : ""}` : "")
    : (d.runtime ? `${Math.floor(d.runtime / 60) ? `${Math.floor(d.runtime / 60)}h ` : ""}${d.runtime % 60}m` : "");
  const meta = [year, genres.join(", "), length].filter(Boolean).join(" · ");
  const cast = ((d.credits && d.credits.cast) || []).slice(0, 5).map((c) => c.name);
  const makers = kind === "tv"
    ? (d.created_by || []).map((c) => c.name)
    : ((d.credits && d.credits.crew) || []).filter((c) => c.job === "Director").map((c) => c.name);
  const slug = `${kind}-${id}`;
  const signIn = `/login?next=${encodeURIComponent(`/browse?title=${slug}`)}`;
  const picture = d.backdrop_path
    ? `<img src="${TMDB_IMAGES}/w1280${d.backdrop_path}" srcset="${TMDB_IMAGES}/w780${d.backdrop_path} 780w, ${TMDB_IMAGES}/w1280${d.backdrop_path} 1280w" sizes="100vw" alt="" fetchpriority="high" decoding="async">`
    : "";
  const like = ((d.recommendations && d.recommendations.results) || [])
    .filter((r) => r.poster_path && (r.title || r.name) && (r.media_type === "movie" || r.media_type === "tv"))
    .slice(0, 12)
    .map((r) => `<li><a href="/title/${r.media_type}-${r.id}"><img src="${TMDB_IMAGES}/w185${r.poster_path}" alt="" loading="lazy" decoding="async" width="185" height="278">${html(r.title || r.name)}</a></li>`)
    .join("");
  return `<div id="first-view">
<style>
#boot{display:none!important}
#first-view{font-family:var(--font-sans,system-ui,sans-serif);color:var(--color-ink,#fff);background:var(--color-title-page,#0f0f0f);min-height:100vh}
#first-view .fv-hero{position:relative;width:100%;aspect-ratio:16/9;max-height:78vh;overflow:hidden;background:var(--color-card,#1e1e1e)}
#first-view .fv-hero img{width:100%;height:100%;object-fit:cover;display:block}
#first-view .fv-hero::after{content:"";position:absolute;inset:0;background:linear-gradient(to top,var(--color-title-page,#0f0f0f) 2%,transparent 55%)}
#first-view .fv-body{position:relative;margin-top:-10vw;padding:0 16px 48px;max-width:980px}
#first-view h1{font-size:clamp(28px,6vw,56px);line-height:1.05;margin:0 0 10px;font-weight:800}
#first-view .fv-meta,#first-view .fv-people{color:var(--color-ink-silver,#bcbcbc);font-size:15px;margin:0 0 14px}
#first-view .fv-play{display:inline-block;background:var(--color-brand,#2f6bff);color:#fff;font-weight:700;padding:10px 22px;border-radius:6px;text-decoration:none;margin:0 0 18px}
#first-view .fv-about{line-height:1.5;font-size:16px;max-width:70ch;margin:0 0 12px}
#first-view .fv-more{padding:0 16px 48px;max-width:980px}
#first-view h2{font-size:20px;margin:30px 0 12px}
#first-view ul{list-style:none;padding:0;margin:0;display:grid;grid-template-columns:repeat(auto-fill,minmax(104px,1fr));gap:12px}
#first-view li a{color:var(--color-ink-soft,#e5e5e5);text-decoration:none;font-size:13px;line-height:1.3}
#first-view li img{width:100%;height:auto;aspect-ratio:2/3;object-fit:cover;border-radius:4px;display:block;background:var(--color-card,#1e1e1e);margin-bottom:6px}
</style>
<main><article><div class="fv-hero">${picture}</div>
<div class="fv-body">
<h1>${html(name)}</h1>
${meta ? `<p class="fv-meta">${html(meta)}</p>` : ""}
<a class="fv-play" href="${attr(signIn)}">Sign in to watch</a>
${d.overview ? `<p class="fv-about">${html(d.overview)}</p>` : ""}
${cast.length ? `<p class="fv-people">Starring: ${html(cast.join(", "))}</p>` : ""}
${makers.length ? `<p class="fv-people">${kind === "tv" ? "Created by" : "Directed by"}: ${html(makers.slice(0, 3).join(", "))}</p>` : ""}
</div></article>
${like ? `<nav class="fv-more" aria-label="More like this"><h2>More like this</h2><ul>${like}</ul></nav>` : ""}
</main>
</div>`;
}

/** The landing page as plain HTML while the app downloads, in the site's own English words (i18n/en.ftl, read from the deployment, so
 *  nothing is written twice): the header, the hero, the reasons to join and the questions with their answers. A returning visitor (a
 *  stored session or chosen profile, as `is_returning` in src/services/account/sign_in.rs) never sees it: the app sends them to Home. */
async function landingFirstView(env, origin) {
  const res = await env.ASSETS.fetch(new Request(`${origin}/i18n/en.ftl`));
  if (!res.ok) return null;
  const words = ftl(await res.text());
  const t = (k) => html((words[k] || "").replace(/\{\s*\$brand\s*\}/g, BRAND));
  if (!words["login-unlimited-films-series-and-more"]) return null;
  // the same reasons and questions as src/pages/landing/website.rs
  const reasons = ["screens", "hd", "subs", "place"]
    .map((r) => `<li><h3>${t(`login-reason-${r}-title`)}</h3><p>${t(`login-reason-${r}-body`)}</p></li>`).join("");
  const faq = ["what-is", "free-to-use", "need-account", "watch-on-tv", "sources", "kids-profile"]
    .filter((s) => words[`help-art-${s}-title`])
    .map((s) => `<li><h3><a href="/help/article/${s}">${t(`help-art-${s}-title`)}</a></h3><p>${t(`help-art-${s}-summary`)}</p></li>`).join("");
  return `<div id="first-view">
<script>try{if(localStorage.getItem("streams_session")||sessionStorage.getItem("streams_session")||localStorage.getItem("streams_active_profile"))document.getElementById("first-view").remove()}catch(e){}</script>
<style>
#boot{display:none!important}
#first-view{font-family:var(--font-sans,system-ui,sans-serif);color:var(--color-ink,#fff);background:var(--color-void,#000);min-height:100vh}
#first-view .fv-nav{display:flex;justify-content:space-between;align-items:center;height:88px;padding:0 24px}
#first-view .fv-nav img{height:28px;width:auto}
#first-view .fv-in,#first-view .fv-cta{background:var(--color-brand,#2f6bff);color:#fff;font-weight:700;text-decoration:none;border-radius:999px}
#first-view .fv-in{padding:8px 18px;font-size:14px}
#first-view .fv-hero{min-height:60vh;display:flex;flex-direction:column;align-items:center;justify-content:center;text-align:center;padding:24px 24px 64px}
#first-view h1{font-size:clamp(32px,6vw,56px);line-height:1.1;margin:0;font-weight:800;max-width:18ch}
#first-view .fv-free{font-size:20px;font-weight:500;margin:16px 0 0}
#first-view .fv-ready{font-size:16px;margin:28px 0 16px;max-width:60ch}
#first-view .fv-cta{display:inline-block;padding:14px 28px;font-size:18px}
#first-view section{padding:24px;max-width:1100px;margin:0 auto}
#first-view h2{font-size:24px;margin:0 0 16px}
#first-view ul{list-style:none;padding:0;margin:0;display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:12px}
#first-view li{background:var(--color-card,#1e1e1e);border-radius:12px;padding:20px}
#first-view h3{font-size:18px;margin:0 0 8px}
#first-view h3 a{color:inherit}
#first-view li p{margin:0;color:var(--color-ink-body,#b3b3b3);line-height:1.5}
</style>
<header class="fv-nav"><a href="/"><img src="/assets/logos/streams-wordmark-white.svg" alt="${BRAND}" width="120" height="28"></a><a class="fv-in" href="/login">${t("login-sign-in")}</a></header>
<main><div class="fv-hero">
<h1>${t("login-unlimited-films-series-and-more")}</h1>
<p class="fv-free">${t("auth-tick-free")}</p>
<p class="fv-ready">${t("login-ready-to-watch-enter-your")}</p>
<a class="fv-cta" href="/signup">${t("login-get-started")}</a>
</div>
<section><h2>${t("login-more-reasons")}</h2><ul>${reasons}</ul></section>
${faq ? `<section><h2>${t("landing-faq")}</h2><ul>${faq}</ul></section>` : ""}
</main>
</div>`;
}

/** A Help Centre page as plain HTML while the app downloads, in the site's English words (i18n/en.ftl): an article's title, summary and
 *  text (paragraphs, numbered and plain lists, bold, links), or the home page's list of every article with its summary (the articles
 *  of the help host's sitemap). Null for an article that does not exist (the app then decides). */
async function helpFirstView(env, origin, slug) {
  const res = await env.ASSETS.fetch(new Request(`${origin}/i18n/en.ftl`));
  if (!res.ok) return null;
  const words = ftlBlocks(await res.text());
  const fill = (s) => (s || "").replace(/\{\s*\$(\w+)\s*\}/g, (_, v) => HELP_VARS[v] ?? "");
  const style = `<style>
#boot{display:none!important}
#first-view{font-family:var(--font-sans,system-ui,sans-serif);color:var(--color-ink,#fff);background:var(--color-page,#141414);min-height:100vh;padding:24px 16px 64px}
#first-view main{max-width:760px;margin:0 auto}
#first-view nav a{color:var(--color-ink-body,#b3b3b3);text-decoration:none;font-size:14px}
#first-view h1{font-size:clamp(26px,5vw,40px);line-height:1.15;margin:16px 0 12px;font-weight:800}
#first-view .fv-lead{font-size:18px;color:var(--color-ink-soft,#e5e5e5);margin:0 0 20px}
#first-view p,#first-view li{line-height:1.6;font-size:16px;color:var(--color-ink-soft,#e5e5e5)}
#first-view a{color:var(--color-brand-glow,#7aa6ff)}
#first-view ul.fv-list{list-style:none;padding:0}
#first-view ul.fv-list li{padding:12px 0;border-bottom:1px solid var(--color-line,#333)}
#first-view ul.fv-list a{font-weight:700;text-decoration:none}
#first-view ul.fv-list span{display:block;color:var(--color-ink-body,#b3b3b3);font-size:14px}
</style>`;
  if (slug) {
    const title = fill(words[`help-art-${slug}-title`]);
    if (!title) return null;
    const summary = fill(words[`help-art-${slug}-summary`]);
    const body = helpBody(fill(words[`help-art-${slug}-body`]));
    return {
      title, description: summary,
      html: `<div id="first-view">${style}<main><nav><a href="/help">${html(`${BRAND} Help Centre`)}</a></nav>
<article><h1>${html(title)}</h1>${summary ? `<p class="fv-lead">${html(summary)}</p>` : ""}${body}</article></main></div>`,
    };
  }
  // the home: every article the help host's sitemap lists, with its summary
  const map = await env.ASSETS.fetch(new Request(`${origin}/sitemap.xml`)).then((r) => (r.ok ? r.text() : "")).catch(() => "");
  const slugs = [...new Set([...map.matchAll(/\/help\/article\/([a-z0-9-]+)</g)].map((m) => m[1]))];
  const items = slugs.filter((s) => words[`help-art-${s}-title`])
    .map((s) => `<li><a href="/help/article/${s}">${html(fill(words[`help-art-${s}-title`]))}</a><span>${html(fill(words[`help-art-${s}-summary`]))}</span></li>`)
    .join("");
  if (!items) return null;
  const page = PAGES["/help"];
  return {
    title: page[0], description: page[1],
    html: `<div id="first-view">${style}<main><h1>${html(`${BRAND} ${page[0]}`)}</h1><p class="fv-lead">${html(page[1])}</p><ul class="fv-list">${items}</ul></main></div>`,
  };
}

// the policies by address: their id in i18n/en.ftl and number of sections (src/data/help.rs, LEGAL)
const LEGAL = {
  "/help/legal/privacy": ["privacy", 14], "/help/legal/terms": ["terms", 18], "/help/legal/cookies": ["cookie-policy", 4],
  "/help/legal/dmca": ["dmca", 6], "/help/legal/disclaimer": ["disclaimer", 5], "/help/legal/credits": ["credits", 6],
};

/** A policy as plain HTML while the app downloads, in the order src/pages/help/legal.rs shows it: title, date, each numbered section
 *  with its text and points, then the other policies. The texts keep their <strong> and their links to outside sites. */
async function legalFirstView(env, origin, path) {
  const res = await env.ASSETS.fetch(new Request(`${origin}/i18n/en.ftl`));
  if (!res.ok) return null;
  const words = ftl(await res.text());
  const [id, sections] = LEGAL[path];
  const fill = (s) => (s || "").replace(/\{\s*\$(\w+)\s*\}/g, (_, v) => HELP_VARS[v] ?? "");
  const inline = (s) => html(fill(s))
    .replace(/&lt;(\/?)strong&gt;/g, "<$1strong>")
    .replace(/&lt;a href="(https:\/\/[^"\s]+)"[^&]*&gt;/g, (_, u) => `<a href="${attr(u)}" rel="noopener">`).replace(/&lt;\/a&gt;/g, "</a>");
  const title = fill(words[`legal-${id}-title`]);
  if (!title) return null;
  let body = "";
  for (let n = 1; n <= sections; n++) {
    const points = [];
    for (let k = 1; words[`legal-${id}-${n}-b${k}`]; k++) points.push(`<li>${inline(words[`legal-${id}-${n}-b${k}`])}</li>`);
    body += `<section><h2>${inline(words[`legal-${id}-${n}-title`])}</h2><p>${inline(words[`legal-${id}-${n}-body`])}</p>${points.length ? `<ul>${points.join("")}</ul>` : ""}</section>`;
  }
  const others = Object.entries(LEGAL).filter(([p]) => p !== path)
    .map(([p, [o]]) => `<li><a href="${p}">${html(fill(words[`legal-${o}-title`]))}</a></li>`).join("");
  return `<div id="first-view"><style>
#boot{display:none!important}
#first-view{font-family:var(--font-sans,system-ui,sans-serif);color:var(--color-ink,#fff);background:var(--color-page,#141414);min-height:100vh;padding:24px 16px 64px}
#first-view main{max-width:760px;margin:0 auto}
#first-view nav a{color:var(--color-ink-body,#b3b3b3);text-decoration:none;font-size:14px}
#first-view h1{font-size:clamp(26px,5vw,40px);line-height:1.15;margin:16px 0 8px;font-weight:800}
#first-view h2{font-size:20px;margin:32px 0 8px}
#first-view .fv-date{color:var(--color-ink-body,#b3b3b3);font-weight:600;margin:0 0 16px}
#first-view p,#first-view li{line-height:1.6;font-size:16px;color:var(--color-ink-soft,#e5e5e5)}
#first-view a{color:var(--color-brand-glow,#7aa6ff)}
</style><main><nav><a href="/help">${html(`${BRAND} Help Centre`)}</a></nav>
<article><h1>${html(title)}</h1><p class="fv-date">${html(fill(words[`legal-${id}-updated`]))}</p>${body}</article>
<nav aria-label="Other policies"><h2>${html(fill(words["help-ui-other-policies"]) || "Other policies")}</h2><ul>${others}</ul></nav></main></div>`;
}

// the values src/pages/help/mod.rs fills into help texts (src/brand.rs)
const HELP_VARS = { brand: BRAND, help_email: "help@streams.watch", contribute_email: "contribute@streams.watch", support_email: "help@streams.watch" };

/** A help article's text as HTML: blocks split at blank lines; a block of "1." lines is a numbered list, of "- " lines a plain list,
 *  anything else a paragraph; <b> and the app's link form <a=address> are kept, everything else is escaped. */
function helpBody(text) {
  const inline = (s) => html(s)
    .replace(/&lt;b&gt;/g, "<b>").replace(/&lt;\/b&gt;/g, "</b>")
    .replace(/&lt;a=([^\s&]+)&gt;/g, (_, u) => `<a href="${attr(u)}">`).replace(/&lt;\/a&gt;/g, "</a>");
  return text.split(/\n\s*\n/).map((block) => {
    const lines = block.split("\n").map((l) => l.trim()).filter(Boolean);
    if (!lines.length) return "";
    // a "## " line heads the rest of its block
    if (/^##\s/.test(lines[0])) return `<h2>${inline(lines[0].replace(/^##\s*/, ""))}</h2>${lines.length > 1 ? helpBody(lines.slice(1).join("\n")) : ""}`;
    if (lines.every((l) => /^\d+\.\s/.test(l))) return `<ol>${lines.map((l) => `<li>${inline(l.replace(/^\d+\.\s*/, ""))}</li>`).join("")}</ol>`;
    if (lines.every((l) => /^[-*]\s/.test(l))) return `<ul>${lines.map((l) => `<li>${inline(l.replace(/^[-*]\s*/, ""))}</li>`).join("")}</ul>`;
    return `<p>${inline(lines.join(" "))}</p>`;
  }).join("");
}

/** Every message of a Fluent file with its lines kept (a multi-line body keeps its blank lines and line starts), by key. */
function ftlBlocks(text) {
  const out = {};
  let key = null, lines = [];
  const close = () => { if (key) out[key] = lines.join("\n").replace(/\n+$/, "").trim(); };
  for (const line of text.split("\n")) {
    const m = /^([a-z][a-z0-9-]*)\s*=\s?(.*)$/.exec(line);
    if (m) { close(); key = m[1]; lines = [m[2]]; continue; }
    if (key && (line.trim() === "" || /^\s+\S/.test(line)) && !/^\s+\./.test(line)) { lines.push(line.replace(/^ {4}/, "")); continue; }
    close(); key = null; lines = [];
  }
  close();
  return out;
}

/** The single and continued line messages of a Fluent file, by key (comments, terms and attributes left out). */
function ftl(text) {
  const out = {};
  let key = null;
  for (const line of text.split("\n")) {
    const m = /^([a-z][a-z0-9-]*)\s*=\s?(.*)$/.exec(line);
    if (m) { key = m[1]; out[key] = m[2].trim(); }
    else if (key && /^\s+\S/.test(line) && !/^\s+\./.test(line)) out[key] = `${out[key]} ${line.trim()}`.trim();
    else key = null;
  }
  return out;
}

/** schema.org people from names (undefined when there are none, so the field is left out). */
function person(names) {
  return names.length ? names.map((name) => ({ "@type": "Person", name })) : undefined;
}

/** `text` cut at a word before `max` characters, with an ellipsis when it was cut. */
function shorten(text, max) {
  const t = text.replace(/\s+/g, " ").trim();
  if (t.length <= max) return t;
  const cut = t.slice(0, max - 1);
  return `${cut.slice(0, Math.max(cut.lastIndexOf(" "), max / 2)).replace(/[\s,;:.]+$/, "")}…`;
}

/** The response with its head's title, description, canonical address, preview tags and schema.org data replaced where given, and the
 *  first view (if any) at the top of the body. */
function withHead(res, h, firstView) {
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
  if (firstView) rw = rw.on("body", { element(e) { e.prepend(firstView, { html: true }); } });
  const out = rw.transform(res);
  // the body's length changed with the head
  const headers = new Headers(out.headers);
  headers.delete("content-length");
  return new Response(out.body, { status: out.status, headers });
}

function attr(s) {
  return String(s).replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;");
}

function html(s) {
  return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
