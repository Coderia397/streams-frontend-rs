"""Collects poster paths per genre from the live engine (feed rows) into posters.json for build.py. Uses the test account (see the notes) for the token."""
import json,urllib.request,time,os
SB="https://emvyjhleyhimxxdqybsn.supabase.co"; KEY="sb_publishable_m286TDvNziieAgk--uDLlQ_MJQHgxNV"
r=urllib.request.Request(SB+"/auth/v1/token?grant_type=password",data=json.dumps({"email":"streams.test.user@example.com","password":os.environ["STREAMS_TEST_PASSWORD"]}).encode(),headers={"apikey":KEY,"Content-Type":"application/json"})
tok=json.load(urllib.request.urlopen(r))["access_token"]
B="https://ibrahimar397-streams-engine.hf.space/gradio_api/engine"
def get(p):
    return json.load(urllib.request.urlopen(urllib.request.Request(B+p,headers={"Authorization":"Bearer "+tok}),timeout=90))
out={}
def add(cat,items):
    for it in items:
        pp=it.get("poster_path"); 
        if not pp: continue
        t=it.get("title") or it.get("name")
        out.setdefault(cat,{})[pp]=t

import urllib.parse, os
cats={"popular":None,"drama":"Drama","comedy":"Comedy","action":"Action","animation":"Animation","family":"Family","scifi":"Science Fiction","romance":"Romance","thriller":"Thriller","fantasy":"Fantasy","horror":"Horror","kids":"Family"}
res={}
for c,g in cats.items():
    seen={}
    for off in (0,12,24):
        for sid in ("a","b") if c=="popular" else ("a",):
            q=f"/api/feed?limit=12&offset={off}&session_id=wall-{c}-{sid}"+(f"&genre={urllib.parse.quote(g)}" if g else "")
            try: f=get(q)
            except Exception: continue
            for row in f.get("rows",[]):
                for it in row.get("items",[]):
                    pp=it.get("poster_path")
                    if pp and pp not in seen: seen[pp]={"p":pp,"t":it.get("title") or it.get("name"),"y":it.get("release_year") or "","v":it.get("vote_average") or 0,"r":row.get("title")}
    res[c]=[x for x in seen.values() if x["v"]>=6.9]
json.dump(res,open(os.environ.get("WALLS_WORK","/tmp/streams-walls/")+"posters.json","w"))
