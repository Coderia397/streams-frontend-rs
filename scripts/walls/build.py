"""Renders the wall-of-posters backgrounds of the landing page (assets/landing/walls) from real catalogue posters, in several layouts and genre sets.
  uv run --with websockets python scripts/walls/build.py [variant ...]        then convert the PNGs to WebP (900 x 2000 phone, 1600 x 900 wide, quality 80)
Needs Chrome (path below) and posters.json from collect_posters.py."""
import json, random, asyncio, subprocess, time, urllib.request, base64, sys, os
import websockets
SC=os.environ.get("WALLS_WORK", "/tmp/streams-walls/")  # work folder; needs posters.json (made by collect_posters.py) and a walls/ subfolder
P=json.load(open(SC+"posters.json"))
IMG="https://image.tmdb.org/t/p/w342"
def pick(pools,seed,n):
    rnd=random.Random(seed); items={}
    for pool in pools:
        for x in sorted(P[pool],key=lambda x:-x["v"])[:200]: items.setdefault(x["p"],x)
    lst=list(items.values()); rnd.shuffle(lst); return lst[:n]
def html(variant,w,h,tile,pools,seed):
    layout=variant["layout"]; rnd=random.Random(seed)
    gap=int(tile*0.05)
    th=int(tile*1.5)
    cols=int((max(w,h)*2.1)/(tile+gap))+2
    rows=int((max(w,h)*2.1)/(th+gap))+3
    n=cols*rows
    items=pick(pools,seed,n)
    while len(items)<n: items+=items
    items=items[:n]
    colhtml=[]
    for c in range(cols):
        off=-rnd.randint(0,th)
        cells="".join(f'<img src="{IMG}{items[c*rows+r]["p"]}" style="width:{tile}px;height:{th}px">' for r in range(rows))
        colhtml.append(f'<div class="col" style="margin-top:{off}px">{cells}</div>')
    if layout=="mosaic":
        # every third poster in a column is large (two tiles wide) by mixing a second set of tall columns
        pass
    rot=variant.get("rot",-14); persp=variant.get("persp",False)
    tr=f"perspective(1600px) rotateX(28deg) rotateZ({rot}deg) scale(1.05)" if persp else f"rotate({rot}deg)"
    return f'''<!doctype html><meta charset=utf-8><style>
html,body{{margin:0;background:#000;width:{w}px;height:{h}px;overflow:hidden}}
.wrap{{position:absolute;left:50%;top:50%;transform:translate(-50%,-50%) {tr};transform-origin:50% 50%;display:flex;gap:{gap}px;width:{cols*(tile+gap)}px}}
.col{{display:flex;flex-direction:column;gap:{gap}px;flex:none}}
.col img{{border-radius:{int(tile*0.045)}px;object-fit:cover;background:#16161c;display:block;box-shadow:0 6px 20px rgba(0,0,0,.45)}}
</style><div class="wrap">{"".join(colhtml)}</div>'''
VARIANTS=[
 ("diagonal-popular",   dict(layout="diag",rot=-14),               ["popular","drama","action"],      300),
 ("perspective-cinema", dict(layout="persp",rot=-16,persp=True),   ["popular","drama","thriller"],    300),
 ("diagonal-animation", dict(layout="diag",rot=-10),               ["animation","family"],            320),
 ("columns-scifi",      dict(layout="cols",rot=-7),                ["scifi","action","fantasy"],      310),
 ("perspective-romcom", dict(layout="persp",rot=-14,persp=True),   ["romance","comedy"],              300),
 ("diagonal-noir",      dict(layout="diag",rot=-16),               ["thriller","horror","drama"],     300),
 ("diagonal-kids",      dict(layout="diag",rot=-12),               ["kids","family","animation"],     330),
 ("columns-fantasy",    dict(layout="cols",rot=-8),                ["fantasy","scifi","animation"],   310),
]
SIZES={"phone":(1080,2400,1.0),"wide":(1920,1080,0.92)}
CH="/Applications/Google Chrome Beta.app/Contents/MacOS/Google Chrome Beta"; PORT=9531
async def main(only=None):
    proc=subprocess.Popen([CH,"--headless=new","--disable-gpu",f"--remote-debugging-port={PORT}","--window-size=1920,1100","--user-data-dir=/private/tmp/claude-501/chrome-walls","about:blank"],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    time.sleep(4)
    ws_url=[t for t in json.loads(urllib.request.urlopen(f"http://localhost:{PORT}/json").read()) if t["type"]=="page"][0]["webSocketDebuggerUrl"]
    try:
        async with websockets.connect(ws_url,max_size=200_000_000) as ws:
            n=0
            async def call(m,p=None):
                nonlocal n; n+=1
                await ws.send(json.dumps({"id":n,"method":m,"params":p or {}}))
                while True:
                    r=json.loads(await ws.recv())
                    if r.get("id")==n: return r.get("result",r)
            await call("Page.enable")
            for name,var,pools,tile in VARIANTS:
                if only and name not in only: continue
                for sz,(w,h,k) in SIZES.items():
                    seed=abs(hash(name))%1000 if False else sum(map(ord,name))*7+(1 if sz=="wide" else 0)
                    t=int(tile*k)
                    path=SC+f"walls/{name}-{sz}.html"; open(path,"w").write(html(var,w,h,t,pools,seed))
                    await call("Emulation.setDeviceMetricsOverride",{"width":w,"height":h,"deviceScaleFactor":1,"mobile":False})
                    await call("Page.navigate",{"url":"file://"+path})
                    # wait for images
                    for _ in range(60):
                        r=await call("Runtime.evaluate",{"expression":"[...document.images].filter(i=>!i.complete).length+'/'+document.images.length","returnByValue":True})
                        left=int(r["result"]["value"].split("/")[0])
                        if left==0: break
                        await asyncio.sleep(1)
                    await asyncio.sleep(1)
                    r=await call("Page.captureScreenshot",{"format":"png"})
                    open(SC+f"walls/{name}-{sz}.png","wb").write(base64.b64decode(r["data"]))
                    print(name,sz,"imgs left",left,flush=True)
    finally: proc.terminate()
asyncio.run(main(sys.argv[1:] or None))
