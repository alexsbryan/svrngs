"""Turns the demo's faults on and off, as an operator would in flagd's UI:
by editing the flag file flagd watches. The agents never see this.
  python3 -I flags.py reset | set <flag> <variant> | show
"""
import json, sys
F = "vendor-cache/src/src/flagd/demo.flagd.json"
d = json.load(open(F))
def save(): json.dump(d, open(F, "w"), indent=2)
cmd = sys.argv[1]
if cmd == "reset":
    for f in d["flags"].values():
        f["defaultVariant"] = "off" if "off" in f["variants"] else f["defaultVariant"]
    d["flags"]["productCatalogFailure"]["targeting"] = {"if": [{"==": [{"var": "product_id"}, "OLJCESPC7Z"]}, "off", "off"]}
    save()
elif cmd == "set":
    flag, variant = sys.argv[2], sys.argv[3]
    f = d["flags"][flag]; assert variant in f["variants"], f["variants"]
    if flag == "productCatalogFailure":
        f["targeting"] = {"if": [{"==": [{"var": "product_id"}, "OLJCESPC7Z"]}, variant, "off"]}
    else:
        f["defaultVariant"] = variant
    save()
print(json.dumps({k: (v.get("targeting") if k == "productCatalogFailure" else v["defaultVariant"]) for k, v in d["flags"].items() if (v.get("targeting") if k == "productCatalogFailure" else v["defaultVariant"]) not in ("off", None) or k == "productCatalogFailure"}))
