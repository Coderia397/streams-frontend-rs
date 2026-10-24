"""Tells the IndexNow search engines (Bing, Yandex, Naver, Seznam, Yep; not Google) about the site's pages: every address in the live
sitemaps, in batches of 10,000, with the key file the site serves (public/<key>.txt). Run after a deploy that adds or changes pages.
    python3 -I scripts/indexnow.py            # prints what it would send
    python3 -I scripts/indexnow.py --send"""
import json, re, sys, urllib.request

HOST = "streams.watch"
KEY = "26ed9fc21105876fd210922caba18a27"
SITEMAPS = [f"https://{HOST}/sitemap.xml", f"https://{HOST}/sitemap-titles.xml"]


def addresses():
    out = []
    for sm in SITEMAPS:
        text = urllib.request.urlopen(sm, timeout=30).read().decode("utf-8", "ignore")
        out += [u for u in re.findall(r"<loc>\s*([^<\s]+)\s*</loc>", text) if u.startswith(f"https://{HOST}/")]
    return list(dict.fromkeys(out))


def main():
    # the key file must be live first, or every submission is refused
    live = urllib.request.urlopen(f"https://{HOST}/{KEY}.txt", timeout=15).read().decode().strip()
    if live != KEY:
        sys.exit(f"the key file at https://{HOST}/{KEY}.txt does not hold the key yet: deploy first")
    urls = addresses()
    print(len(urls), "addresses")
    for i in range(0, len(urls), 10000):
        body = {"host": HOST, "key": KEY, "keyLocation": f"https://{HOST}/{KEY}.txt", "urlList": urls[i:i + 10000]}
        if "--send" not in sys.argv:
            print("would send", len(body["urlList"]), "addresses, first", body["urlList"][0])
            continue
        req = urllib.request.Request("https://api.indexnow.org/indexnow", data=json.dumps(body).encode(),
                                     headers={"Content-Type": "application/json; charset=utf-8"})
        with urllib.request.urlopen(req, timeout=60) as r:
            print("sent", len(body["urlList"]), "->", r.status)


if __name__ == "__main__":
    main()
