#!/usr/bin/env python3
"""tranche.py — a steering pass's table of a tranche's workers, from lore.

For each item, the worker's session in its lane well
(`…/.claude/worktrees/att-<item>`) is read through `lore trace --steps`
and reduced to the numbers a pass compares tranche to tranche:

  price      list USD, at the rate table of the day it is read
  requests   API requests
  peak       the deepest context of any request (input + cache read + cache
             write), in k tokens
  deep       the share of the price spent by requests above --deep k
  clock      first request to last, in minutes
  waiting    the minutes inside gaps longer than --gap seconds between
             consecutive requests — the machine's half of a landing
  on         what the last instruction before each gap was: a capture, a
             gate, a suite, the emulator or other — the same classes the
             nineteenth and twentieth passes read by hand

The nineteenth pass measured this shape once by hand, the twentieth
again by a scratch script, and the twenty-first is the third reach — so
it lives here (`CLAUDE.md`, "A probe shape reached for a third time
graduates into tools/"). Nothing here is inference: every number is a
transcript field `lore` already carries, folded.

    python3 tools/tranche.py 1214 1228 1243            # workers by item
    python3 tools/tranche.py --session 7ecd9b33        # one session by id
    python3 tools/tranche.py --json 1214 …             # the rows, for a table

The classes are read off the Bash command's text, and `--show-gaps`
prints each gap with its command so a class can be checked by eye.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from datetime import datetime

WELL = "worktrees-att-{item}"

# What a worker waits on, by the last instruction before the gap. Order
# matters: a gate runs the suite, so the gate's own line is tested first.
CLASSES = [
    ("gate", re.compile(r"release_gate\.py")),
    ("capture", re.compile(
        r"waitrun\.sh|unattended_capture|longtrace\.sh|startcapture\.sh|"
        r"viadriver|clickdriver\.sh|winelaunch|golden/.*\.sh|frame_snapshot|"
        r"chain\d*\.log|cap\w*\.log|pgrep.*capture|[Cc]apture run|Logs")),
    ("emulator", re.compile(r"callfn\.py|step4\.py|lift\.py")),
    ("suite", re.compile(r"cargo (test|clippy|build|run)|guard\.sh|offline_tests|memcap\.sh|mut\w*\.(sh|log)")),
    # A status line to the commander, then nothing: the wait is on whatever
    # the worker had backgrounded, and the message says which.
    ("status", re.compile(r'"to":"attrition"')),
    # A script in the job's tmp (parked 1421): the frame's own row for a
    # command the worktree guard refuses, so its text names no gate or
    # suite — `--show-gaps` reads the script's name.
    ("script", re.compile(r"(?:\$CLAUDE_JOB_DIR|jobs/[^/\s]+)/tmp/\S+\.(?:sh|py)\b")),
]


def classify(instruction: dict | None) -> str:
    if not instruction:
        return "other"
    # A fanned batch's subagents (1619: 56 of 63 minutes): the wait is on
    # the Agent tool, whatever its prompt says.
    if instruction.get("tool") == "Agent":
        return "subagent"
    text = instruction.get("input") or ""
    for name, rx in CLASSES:
        if rx.search(text):
            return name
    return "other"


def ts(s: str) -> datetime:
    return datetime.fromisoformat(s.replace("Z", "+00:00"))


def lore(*args: str) -> dict:
    out = subprocess.run(["lore", *args, "--format", "json"], capture_output=True, text=True, check=True)
    return json.loads(out.stdout)


def sessions_of(item: int) -> list[str]:
    d = lore("sessions", "--well", WELL.format(item=item), "--limit", "20")
    return [s["sessionId"] for s in d.get("sessions", [])]


def trace(session: str) -> dict:
    return lore("trace", session, "--steps", "--limit", "5000", "--head", "200")


def fold(tr: dict, gap_s: int, deep_k: int) -> dict:
    """Reduce one `lore trace --steps` to the row a pass reads."""
    requests: list[dict] = []
    instructions: list[dict] = []
    # The turn each request opened, by kind: a gap that ends at a `relay`
    # turn was a wait on another session's message (parked 1421: 1398's
    # 334 minutes for a commander's reply read as the suite).
    opened_by: dict[str, str] = {}
    for tx in tr.get("transactions", []):
        reqs = tx.get("requests")
        if isinstance(reqs, list):
            requests.extend(reqs)
            if reqs:
                opened_by[min(r["ts"] for r in reqs)] = tx.get("kind") or ""
        instructions.extend(tx.get("instructions") or [])
    requests.sort(key=lambda r: r["ts"])
    instructions.sort(key=lambda i: i["ts"])
    price = sum(r.get("listUsd", 0) for r in requests)
    contexts = [r.get("input", 0) + r.get("cacheRead", 0) + r.get("cacheWrite", 0) for r in requests]
    peak = max(contexts, default=0)
    deep = sum(r.get("listUsd", 0) for r, c in zip(requests, contexts) if c > deep_k * 1000)
    waiting = 0.0
    on: dict[str, float] = {}
    gaps: list[tuple[str, float, str]] = []
    for a, b in zip(requests, requests[1:]):
        secs = (ts(b["ts"]) - ts(a["ts"])).total_seconds()
        if secs <= gap_s:
            continue
        # The gap follows an ended turn: the worker backgrounded something
        # and was re-invoked at its exit. The last instruction is usually a
        # read of that task's log, so the class is the nearest instruction
        # before the gap that names a gate, a capture, a suite or the
        # emulator — walked back a dozen instructions, no further.
        before = [ins for ins in instructions if ins["ts"] <= b["ts"]]
        last = before[-1] if before else None
        cls = "other"
        for ins in reversed(before[-12:]):
            c = classify(ins)
            if c != "other":
                cls = c
                break
        if opened_by.get(b["ts"]) == "relay":
            cls = "message"
        waiting += secs / 60
        on[cls] = on.get(cls, 0.0) + secs / 60
        gaps.append((a["ts"], secs / 60, (last or {}).get("input", "")[:120]))
    clock = (ts(requests[-1]["ts"]) - ts(requests[0]["ts"])).total_seconds() / 60 if requests else 0.0
    models = {}
    for r in requests:
        models[r.get("model", "?")] = models.get(r.get("model", "?"), 0) + 1
    return {
        "price": round(price, 2),
        "requests": len(requests),
        "peak_k": round(peak / 1000),
        "deep_share": round(deep / price, 2) if price else 0.0,
        "clock_min": round(clock, 1),
        "waiting_min": round(waiting, 1),
        "working_min": round(clock - waiting, 1),
        "on": {k: round(v, 1) for k, v in sorted(on.items(), key=lambda kv: -kv[1])},
        "models": models,
        "gaps": gaps,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("items", nargs="*", type=int, help="item numbers; each names its lane well")
    ap.add_argument("--session", action="append", default=[], help="a session id or prefix, read directly")
    ap.add_argument("--gap", type=int, default=90, help="seconds; a longer gap between requests is waiting")
    ap.add_argument("--deep", type=int, default=300, help="k tokens; the context above which spend is 'deep'")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--show-gaps", action="store_true")
    args = ap.parse_args()

    rows = []
    for item in args.items:
        sids = sessions_of(item)
        if not sids:
            print(f"{item}: no session in {WELL.format(item=item)}", file=sys.stderr)
            continue
        for sid in sids:
            rows.append({"item": item, "session": sid[:8], **fold(trace(sid), args.gap, args.deep)})
    for sid in args.session:
        rows.append({"item": None, "session": sid[:8], **fold(trace(sid), args.gap, args.deep)})

    if args.json:
        json.dump(rows, sys.stdout, indent=1)
        return 0

    print(f"{'item':>6} {'session':8} {'usd':>7} {'req':>4} {'peak_k':>6} {'deep':>5} {'clock':>6} {'work':>6} {'wait':>6}  on")
    for r in rows:
        on = " ".join(f"{k}={v}" for k, v in r["on"].items())
        print(f"{str(r['item'] or '-'):>6} {r['session']:8} {r['price']:7.2f} {r['requests']:4d} {r['peak_k']:6d} "
              f"{r['deep_share']:5.2f} {r['clock_min']:6.1f} {r['working_min']:6.1f} {r['waiting_min']:6.1f}  {on}")
        if args.show_gaps:
            for at, mins, cmd in r["gaps"]:
                print(f"         {at[11:19]} {mins:6.1f}  {cmd}")
    if len(rows) > 1:
        n = len(rows)
        tot = lambda k: sum(r[k] for r in rows)  # noqa: E731
        on: dict[str, float] = {}
        for r in rows:
            for k, v in r["on"].items():
                on[k] = on.get(k, 0.0) + v
        print(f"{'mean':>6} {'':8} {tot('price')/n:7.2f} {tot('requests')/n:4.0f} {tot('peak_k')/n:6.0f} "
              f"{sum(r['deep_share']*r['price'] for r in rows)/max(tot('price'),1e-9):5.2f} "
              f"{tot('clock_min')/n:6.1f} {tot('working_min')/n:6.1f} {tot('waiting_min')/n:6.1f}  "
              + " ".join(f"{k}={v/n:.1f}" for k, v in sorted(on.items(), key=lambda kv: -kv[1])))
        print(f"{'total':>6} {'':8} {tot('price'):7.2f} {tot('requests'):4d} {max(r['peak_k'] for r in rows):6d} "
              f"{'':5} {tot('clock_min'):6.0f} {tot('working_min'):6.0f} {tot('waiting_min'):6.0f}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
