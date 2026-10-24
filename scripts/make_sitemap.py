#!/usr/bin/env python3
"""Writes public/sitemap.xml from the routes and the help data, so every help article and policy is listed.
Run by the Trunk pre build hook. Reads src/data/help.rs for article slugs and policy paths."""
import re, pathlib
ROOT = pathlib.Path(__file__).resolve().parent.parent
SITE = "https://streams.watch"
data = (ROOT / "src/data/help.rs").read_text()
slugs = re.findall(r'Article \{ slug: "([a-z0-9-]+)"', data)
docs = re.findall(r'LegalDoc \{ id: "[a-z0-9-]+", path: "([^"]+)"', data)
# only the pages a visitor may open: the rest of the app needs an account and would send a crawler to sign in
pages = ["/", "/login", "/signup", "/help", "/help/contact"] + docs
# no lastmod: a date that is always today tells a search engine nothing (it learns to ignore it), and rewrote the file every day
def url(path, priority):
    return f"  <url><loc>{SITE}{path}</loc><priority>{priority}</priority></url>"
lines = ['<?xml version="1.0" encoding="UTF-8"?>', '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">']
lines += [url(p, "0.9" if p == "/" else "0.7" if p in ("/help", "/signup") else "0.5") for p in pages]
lines += [url(f"/help/article/{s}", "0.5") for s in slugs]
lines.append("</urlset>")
out = ROOT / "public/sitemap.xml"
text = "\n".join(lines) + "\n"
# written only when it changed: a write on every build makes `trunk serve` see a changed file and build again, for ever (and reload the phone each time)
if not out.exists() or out.read_text() != text:
    out.write_text(text)
print(f"sitemap: {len(pages)} pages, {len(slugs)} articles")
