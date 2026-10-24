#!/usr/bin/env python3
"""Writes public/sitemap-titles.xml: the public page of every title with enough of a page to be worth finding (note 117).
Run by hand after the catalog changes (the site's builder has no catalog):
    python3 scripts/make_title_sitemap.py [/Users/ibrahim/streams-catalog/streams_v3.db]
Listed: films and series with at least 50 votes on TMDB, a synopsis and a poster, adult titles left out (34,658 on 2026-10-24),
most voted first, at most 50,000 (the limit of one sitemap file). robots.txt names the file."""
import pathlib, sqlite3, sys
ROOT = pathlib.Path(__file__).resolve().parent.parent
SITE = "https://streams.watch"
DB = sys.argv[1] if len(sys.argv) > 1 else "/Users/ibrahim/streams-catalog/streams_v3.db"
db = sqlite3.connect(f"file:{DB}?mode=ro", uri=True)
rows = db.execute(
    "select media_type, tmdb_id from titles where vote_count >= 50 and coalesce(adult, 0) = 0 "
    "and overview is not null and overview <> '' and poster_path is not null order by vote_count desc limit 50000"
).fetchall()
lines = ['<?xml version="1.0" encoding="UTF-8"?>', '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">']
lines += [f"  <url><loc>{SITE}/title/{'tv' if kind == 'tv' else 'movie'}-{tmdb_id}</loc></url>" for kind, tmdb_id in rows]
lines.append("</urlset>")
out = ROOT / "public/sitemap-titles.xml"
text = "\n".join(lines) + "\n"
if not out.exists() or out.read_text() != text:
    out.write_text(text)
print(f"title sitemap: {len(rows)} titles, {len(text) / 1e6:.1f} MB")
