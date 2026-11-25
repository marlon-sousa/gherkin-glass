# Turns feature files into the spike's store: blocks by hash, and one list per version.
import sys, os, json, hashlib
from gherkin.parser import Parser
S = sys.argv[1]; out = sys.argv[2]
VERSIONS = [("v1.0.0", "v1"), ("v1.1.0", "v2")]
store = {}
def cj(o): return json.dumps(o, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
def put(block):
    h = hashlib.sha256(cj(block).encode()).hexdigest()[:16]
    store[h] = block; return h
def desc(d):
    return "\n".join(l.strip() for l in (d or "").split("\n")).strip()
def table(t):
    return [[c["value"] for c in r["cells"]] for r in t["rows"]] if t else None
def step(s):
    o = {"keyword": s["keyword"], "text": s["text"]}
    if s.get("dataTable"): o["table"] = table(s["dataTable"])
    if s.get("docString"): o["docString"] = {"mediaType": s["docString"].get("mediaType"), "content": s["docString"]["content"]}
    return o
def tags(b): return [t["name"] for t in b.get("tags", [])]
def common(kind, b):
    return {"kind": kind, "keyword": b["keyword"], "name": b.get("name", ""), "description": desc(b.get("description"))}
manifests = {}
for version, folder in VERSIONS:
    root = f"{S}/spike-src/{folder}"
    search = json.load(open(f"{S}/spike-src/ghg-{folder}/assets/search.json"))
    by_page = {}
    for e in search: by_page.setdefault(e["page"], []).append(e)
    files = []
    for dirpath, _, names in os.walk(root):
        for n in names:
            if n.endswith(".feature"):
                files.append(os.path.relpath(os.path.join(dirpath, n), root))
    files.sort()
    out_files = []
    for path in files:
        doc = Parser().parse(open(f"{root}/{path}", encoding="utf-8").read())
        f = doc["feature"]; page = path[:-len(".feature")]
        entries = iter(by_page[page + ".html"])
        def anchor(kind, name):
            e = next(entries)
            assert e["kind"] == kind and e["name"] == name, (path, kind, name, e)
            return e["anchor"]
        anchor("feature", f["name"])
        fb = common("feature", f); fb["tags"] = tags(f); fb["language"] = f["language"]
        def walk(children):
            items = []
            for c in children:
                if "background" in c:
                    b = c["background"]; blk = common("background", b); blk["steps"] = [step(s) for s in b["steps"]]
                    items.append({"block": put(blk), "anchor": anchor("background", b.get("name", "")), "line": b["location"]["line"]})
                elif "scenario" in c:
                    b = c["scenario"]; outline = bool(b.get("examples"))
                    kind = "scenario-outline" if outline else "scenario"
                    blk = common(kind, b); blk["tags"] = tags(b); blk["steps"] = [step(s) for s in b["steps"]]
                    item = {"anchor": anchor(kind, b.get("name", "")), "line": b["location"]["line"]}
                    if outline:
                        blk["examples"] = [dict(common("examples", x), tags=tags(x), table=([[c["value"] for c in x["tableHeader"]["cells"]]] + [[c["value"] for c in r["cells"]] for r in x.get("tableBody", [])]) if x.get("tableHeader") else None) for x in b["examples"]]
                        item["examples"] = [anchor("examples", x.get("name", "")) for x in b["examples"]]
                    item["block"] = put(blk); items.append(item)
                elif "rule" in c:
                    b = c["rule"]; blk = common("rule", b); blk["tags"] = tags(b)
                    item = {"block": put(blk), "anchor": anchor("rule", b.get("name", "")), "line": b["location"]["line"]}
                    item["children"] = walk(b.get("children", []))
                    items.append(item)
            return items
        children = walk(f.get("children", []))
        out_files.append({"path": path, "page": page, "feature": put(fb), "line": f["location"]["line"], "children": children})
    manifests[version] = {"version": version, "title": "gherkin-glass examples", "files": out_files}
os.makedirs(f"{out}/versions", exist_ok=True)
json.dump(store, open(f"{out}/store.json", "w"), ensure_ascii=False, separators=(",", ":"))
for v, m in manifests.items():
    json.dump(m, open(f"{out}/versions/{v}.json", "w"), ensure_ascii=False, separators=(",", ":"))
json.dump({"title": "gherkin-glass examples", "versions": [v for v, _ in VERSIONS]}, open(f"{out}/versions.json", "w"))
print(len(store), "blocks;", {v: sum(1 for _ in m["files"]) for v, m in manifests.items()})
