#!/usr/bin/env python3
"""passes.py <gamelog> <out.json> -- every DUMP_ALL pass's units, compactly.

The dump nests by indentation (there are no END lines): a `BEGIN X` at
column c opens a block that lasts until the next line at column <= c.
One entry per FULL DUMP pass: frame, seed, every UNITDATA (top-level and
inside ANIMALDATA) with position, orders_x/y, idle/waiting, the order stack
(type + each order's scalar fields, first wins) and each GUY's
type/cur_anim/cur_time/end_time/stopped.
"""
import json
import sys

path, out = sys.argv[1], sys.argv[2]

UNIT_KEYS = {"o", "who", "x_internal", "y_internal", "orders_x", "orders_y",
             "gather_down", "idle", "waiting", "path_recursion", "tolerance",
             "flags", "num_queued", "myhits", "collide", "unit_masks",
             "dest_angle", "angle"}
GUY_KEYS = {"type", "cur_anim", "cur_time", "end_time", "stopped", "x", "y",
            "guy_flags", "(int)variation", "angle", "des_angle"}

passes = []
cur_pass = None
frame = None
stack = []          # (indent, name)
unit = None
guy = None
order = None
build = None
seen_seed = False


def close_to(indent):
    global unit, guy, order, cur_pass, build
    while stack and stack[-1][0] >= indent:
        _, name = stack.pop()
        if name == "UNITDATA":
            unit = None
            order = None
        elif name == "GUY":
            guy = None
        elif name.endswith("ORDER"):
            if not any(n.endswith("ORDER") for _, n in stack):
                order = None
        elif name == "FULL DUMP":
            cur_pass = None
        elif name == "BUILDDATA":
            build = None


with open(path, encoding="latin-1") as f:
    for line in f:
        if not line.strip():
            continue
        indent = len(line) - len(line.lstrip(" "))
        s = line.strip()
        close_to(indent)
        if s.startswith("BEGIN "):
            name = s[6:]
            if name.startswith("FRAME"):
                frame = int(name.split()[1])
                stack.append((indent, name))
                continue
            stack.append((indent, name))
            if name == "FULL DUMP":
                cur_pass = {"frame": frame, "seed": None, "units": [], "builds": 0, "buildlist": []}
                passes.append(cur_pass)
                seen_seed = False
            elif name == "UNITDATA" and cur_pass is not None:
                unit = {"f": {}, "orders": [], "guys": [],
                        "animal": any(n == "ANIMALDATA" for _, n in stack)}
                cur_pass["units"].append(unit)
            elif name == "GUY" and unit is not None:
                guy = {}
                unit["guys"].append(guy)
            elif name.endswith("ORDER") and unit is not None and order is None:
                order = {"type": unit.get("_last_type"), "f": {}}
                unit["orders"].append(order)
            elif name == "BUILDDATA" and cur_pass is not None:
                build = {"f": {}}
                cur_pass["buildlist"].append(build)
                cur_pass["builds"] += 1
            continue
        parts = s.rsplit(" ", 1)
        if len(parts) != 2:
            continue
        k, v = parts
        if cur_pass is not None and not seen_seed and k == "game_random seed":
            cur_pass["seed"] = int(v)
            seen_seed = True
            continue
        top = stack[-1][1] if stack else ""
        if unit is None:
            if build is not None and top in ("BUILDDATA", "OBJECT", "SUBOBJECT"):
                build["f"].setdefault(k, v)
            continue
        if top == "UNITDATA" and k == "type":
            unit["_last_type"] = v
            continue
        if guy is not None:
            if k in GUY_KEYS:
                guy[k] = v
            continue
        if top == "STACK<TYPE>":
            if k == "type":
                order = {"type": v, "f": {}}
                unit["orders"].append(order)
            continue
        if order is not None and top.endswith("ORDER"):
            order["f"].setdefault(k, v)
            continue
        if k in UNIT_KEYS and k not in unit["f"]:
            unit["f"][k] = v

json.dump(passes, open(out, "w"))
print(len(passes), "passes")
for p in passes:
    print(p["frame"], p["seed"], len(p["units"]), "units", p["builds"], "builds")
