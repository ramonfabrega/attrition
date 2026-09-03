#!/usr/bin/env python3
"""Per-frame extraction from Logs/gamelog.txt (DUMP_ALL=1 shape).

track.py KIND FIELDS [--where k=v,...] [--frames a-b] [--file f] [--every n] [--changes]

KIND: UNITDATA | BUILDDATA | CITYDATA | LEADERDATA (the BEGIN name of the record)
FIELDS: comma list of keys; nested GUY fields as guy.x etc.
--where: only records whose fields all match (string compare)
--changes: print a record only when its selected fields differ from its previous frame
Prints: frame <n> | <key=val ...> per matching record.
"""
import sys, os, re
LOGS = os.path.expanduser("~/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/Logs")
DEFAULT = os.path.join(LOGS, "gamelog.txt")
RECORD_KINDS = {"UNITDATA", "BUILDDATA", "CITYDATA", "LEADERDATA", "WALLDATA_TOP", "GUYDATA"}
TRANSPARENT = {"OBJECT", "SUBOBJECT", "WALLDATA"}
SKIP = {"STACK<TYPE>", "BUILDQUEUE", "BUILDQUEUEDATA"}

def parse_args(argv):
    kind = argv[0]; fields = argv[1].split(",")
    opts = {"where": {}, "frames": None, "file": DEFAULT, "every": 1, "changes": False}
    i = 2
    while i < len(argv):
        a = argv[i]
        if a == "--where":
            for kv in argv[i+1].split(","):
                k, v = kv.split("="); opts["where"][k] = v
            i += 2
        elif a == "--frames":
            a_, b_ = argv[i+1].split("-"); opts["frames"] = (int(a_), int(b_)); i += 2
        elif a == "--file":
            f = argv[i+1]; opts["file"] = f if os.path.exists(f) else os.path.join(LOGS, f); i += 2
        elif a == "--every":
            opts["every"] = int(argv[i+1]); i += 2
        elif a == "--changes":
            opts["changes"] = True; i += 1
        else:
            raise SystemExit("bad arg " + a)
    return kind, fields, opts

def records(fh, kind):
    """Yield (frame, record_dict) for every record of `kind` inside BEGIN FRAME blocks."""
    frame = None
    stack = []          # list of (indent, name)
    rec = None; rec_indent = None
    def close():
        nonlocal rec, rec_indent
        if rec is not None:
            r = rec; rec = None; rec_indent = None
            return r
        return None
    for line in fh:
        s = line.rstrip("\n")
        stripped = s.lstrip(" ")
        indent = len(s) - len(stripped)
        if not stripped:
            continue
        if stripped.startswith("BEGIN "):
            name = stripped[6:].strip()
            if name.startswith("FRAME"):
                r = close()
                if r is not None: yield frame, r
                frame = int(name.split()[1]); stack = [(indent, "FRAME")]
                continue
            # pop stack to this indent
            while stack and stack[-1][0] >= indent:
                stack.pop()
            if rec is not None and indent <= rec_indent:
                r = close()
                if r is not None: yield frame, r
            stack.append((indent, name))
            if name == kind and frame is not None:
                rec = {}; rec_indent = indent
            continue
        # key value line
        if rec is None:
            continue
        while stack and stack[-1][0] >= indent:
            stack.pop()
        if indent <= rec_indent:
            r = close()
            if r is not None: yield frame, r
            continue
        # prefix from blocks between the record and here
        inner = [n for (i_, n) in stack if i_ > rec_indent]
        if any(n in SKIP or n.startswith("STACK") for n in inner):
            continue
        named = [n for n in inner if n not in TRANSPARENT]
        parts = stripped.split(" ")
        key = parts[0]; val = " ".join(parts[1:]) if len(parts) > 1 else ""
        if named:
            key = named[-1].lower() + "." + key
        # A record can hold several blocks of the same name — a unit's two
        # GUY entries, say. The first keeps the bare key, so every existing
        # caller is unchanged; the repeats get `key#1`, `key#2`, … and are
        # how a crew figure is read at all.
        if key not in rec:
            rec[key] = val
        else:
            n = 1
            while "%s#%d" % (key, n) in rec:
                n += 1
            rec["%s#%d" % (key, n)] = val
    r = close()
    if r is not None: yield frame, r

def main():
    kind, fields, o = parse_args(sys.argv[1:])
    last = {}
    with open(o["file"], encoding="utf-8", errors="replace") as fh:
        for frame, r in records(fh, kind):
            if o["frames"] and not (o["frames"][0] <= frame <= o["frames"][1]):
                continue
            if frame % o["every"]:
                continue
            if any(r.get(k) != v for k, v in o["where"].items()):
                continue
            vals = [(k, r.get(k, "?")) for k in fields]
            ident = r.get("uid", r.get("o", "?")), r.get("o", "?"), r.get("who", "?")
            if o["changes"]:
                if last.get(ident) == vals:
                    continue
                last[ident] = vals
            print("frame %d | o=%s who=%s | %s" % (frame, ident[1], ident[2], " ".join("%s=%s" % kv for kv in vals)))

if __name__ == "__main__":
    main()
