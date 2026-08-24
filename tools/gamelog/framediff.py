#!/usr/bin/env python3
"""framediff.py run13.json [frame ...] -- per-sim-frame changes, unit by unit.

The block `FRAME n` holds two passes: the end of sim-frame n-1, then the
start of sim-frame n (same seed). Sim-frame n's draws are the LCG steps
between block n's seed and block n+1's; its state change is pass 1 of block
n (== pass 2) to pass 1 of block n+1.
"""
import json
import sys

passes = json.load(open(sys.argv[1]))
want = {int(a) for a in sys.argv[2:]}


def steps(a, b, cap=100000):
    s = a
    for k in range(cap + 1):
        if s == b:
            return k
        s = (s * 0x19660D + 0x3C6EF35F) & 0xFFFFFFFF
    return None


def key(u):
    return ("A" if u["animal"] else "U", int(u["f"]["who"]), int(u["f"]["o"]))


def orders(u):
    out = []
    for o in u["orders"]:
        f = o["f"]
        tag = o["type"]
        bits = []
        for k in ("x", "y", "dest_x", "dest_y", "tile_x", "tile_y", "o", "who", "gather_type", "state", "sub", "frames", "wait", "count", "time"):
            if k in f:
                bits.append(f"{k}={f[k]}")
        out.append(tag + "(" + ",".join(bits) + ")")
    return " ".join(out)


def guys(u):
    return " ".join(
        f"[t{g.get('type')} anim {g.get('cur_anim')} {g.get('cur_time')}/{g.get('end_time')} st{g.get('stopped')}]"
        for g in u["guys"])


blocks = {}
for p in passes:
    if p["frame"] is None:
        continue
    blocks.setdefault(p["frame"], []).append(p)

frames = sorted(blocks)
for a, b in zip(frames, frames[1:]):
    pa, pb = blocks[a][1] if len(blocks[a]) > 1 else blocks[a][0], blocks[b][0]
    n = steps(pa["seed"], pb["seed"])
    if want and a not in want:
        continue
    print(f"=== sim-frame {a}: {n} draws ({pa['seed']:#x} -> {pb['seed']:#x})")
    ua = {key(u): u for u in pa["units"]}
    ub = {key(u): u for u in pb["units"]}
    for k in sorted(set(ua) | set(ub)):
        x, y = ua.get(k), ub.get(k)
        if x is None:
            print(f"  + {k} NEW  {y['f'].get('x_internal')},{y['f'].get('y_internal')} {guys(y)} {orders(y)}")
            continue
        if y is None:
            print(f"  - {k} GONE")
            continue
        ch = []
        for fk in ("x_internal", "y_internal", "orders_x", "orders_y", "idle", "waiting", "gather_down", "path_recursion", "collide", "num_queued"):
            if x["f"].get(fk) != y["f"].get(fk):
                ch.append(f"{fk} {x['f'].get(fk)}->{y['f'].get(fk)}")
        gx, gy = guys(x), guys(y)
        ox, oy = orders(x), orders(y)
        interesting = len(x["guys"]) != len(y["guys"]) or any(
            a.get("cur_anim") != b.get("cur_anim") or a.get("end_time") != b.get("end_time")
            or a.get("stopped") != b.get("stopped") or a.get("type") != b.get("type")
            or int(b.get("cur_time", 0)) != int(a.get("cur_time", 0)) + 1
            for a, b in zip(x["guys"], y["guys"]))
        if interesting:
            ch.append(f"guys {gx} -> {gy}")
        if ox != oy:
            ch.append(f"orders {ox} -> {oy}")
        if ch:
            print(f"  {k}: " + "; ".join(ch))
