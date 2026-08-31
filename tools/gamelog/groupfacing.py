#!/usr/bin/env python3
"""groupfacing.py DUMP — the mirror flag, its writers, and the formations used.

    groupfacing.py gamelog-run45-islands-groups.txt

`docs/GROUPS.md` §6.3 and queue item 23. `Unit::kill_current_order@005e2cb0`
hands the mirror back to the group on a dying move order:

    group.facing = order.facing XOR reversing(leader.angle - order.angle)

and **no run on disk has fired the XOR term**, because `order.facing` has
been 0 at every hand-back anyone has looked at — run31's three clicks are
`0, 0, 0` at the call, the mirrors coming out `0, 0, 1`. The term needs an
order that was *laid out mirrored*, `facing 1`, and then killed.

So the precondition is a single grep, and it is what this reports first:
does any `MOVEORDER` in this dump carry `facing 1` at all. If none does,
the capture has not reached the question and no amount of reading the
group records will change that.

It also reports which **formations** occur (`GROUPDATA.form`: 0 Line, 1
Refused, 2 Envelop, 3 Echelon Right, 4 Echelon Left, -1 none). The mirror
only changes a layout whose slot table reads `reverse` — §6.4's Echelon
rows, `Y = reverse ? Y0 - X : Y0 + X` — so a run in which every group is a
Line can fire the XOR term and still show nothing in the positions.

Needs `GROUPS=1` under `[End Frame]`, which `GameLog::full_dump` gates
`dump_groups` on: without it there is no `GROUPDATA` at all, which is what
run30 discovered by not having any.
"""
import collections
import os
import sys

FORMS = {-1: "none", 0: "Line", 1: "Refused", 2: "Envelop",
         3: "Echelon Right", 4: "Echelon Left"}
ARCHIVE = os.environ.get(
    "RON_GAMELOG_DIR",
    os.path.expanduser(
        "~/Library/Application Support/CrossOver/Bottles/ron/drive_c/users"
        "/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"))


def main():
    path = sys.argv[1]
    if not os.path.isabs(path):
        path = os.path.join(ARCHIVE, path)

    frame = None
    block = None
    stack = []
    groups = 0
    group_facing = collections.Counter()
    forms = collections.Counter()
    order_facing = collections.Counter()
    mirrored_orders = []      # (frame, angle) for every MOVEORDER facing 1
    cur = {}

    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            s = line.strip()
            if s.startswith("BEGIN FRAME "):
                try:
                    frame = int(s.split()[2])
                except (IndexError, ValueError):
                    frame = None
                block, cur, stack = None, {}, []
                continue
            raw = line.rstrip("\r\n")
            indent = len(raw) - len(raw.lstrip(" "))
            if s.startswith("BEGIN "):
                while stack and stack[-1][0] >= indent:
                    stack.pop()
                name = s[6:].strip()
                stack.append((indent, name))
                if name == "GROUPDATA":
                    groups += 1
                cur = {}
                continue
            parts = s.split()
            if len(parts) != 2:
                continue
            # A field belongs to the innermost block indented **less** than it.
            # The dump opens `BEGIN UNITORDER` one deeper than MOVEORDER and
            # then returns to MOVEORDER's own fields at UNITORDER's indent, so
            # tracking only the last `BEGIN` attributes `facing` to UNITORDER
            # and finds no MOVEORDER anywhere.
            while stack and stack[-1][0] >= indent:
                stack.pop()
            block = stack[-1][1] if stack else None
            k, v = parts
            try:
                v = int(v)
            except ValueError:
                continue
            if block == "GROUPDATA":
                if k == "facing":
                    group_facing[v] += 1
                elif k == "form":
                    forms[v] += 1
            elif block == "MOVEORDER":
                cur[k] = v
                if k == "facing":
                    # **Which** MOVEORDER matters. run31 carries `facing 1` on
                    # 123 `EXPLORETOORDER/MOVEORDER` records from frame 245 —
                    # an explore order's move leg, laid out by nothing and
                    # handed back to no group — and on 432
                    # `GroupMoveOrder/MOVEORDER` records from frame 356, which
                    # is its third click, the one §6.3's table gives mirror 1.
                    # Only the second kind is a group layout, so a checker
                    # that counts bare `MOVEORDER` passes a capture that never
                    # reached the question.
                    inside_group = any(n == "GroupMoveOrder" for _, n in stack)
                    order_facing[(inside_group, v)] += 1
                    if v == 1 and inside_group:
                        mirrored_orders.append((frame, cur.get("angle")))

    if groups == 0:
        print("no GROUPDATA in this dump — it was not taken with GROUPS=1,"
              " so the group pool is not a record here (run30's mistake)")
        return 1

    print("GROUPDATA blocks: %d" % groups)
    print("group.facing:  %s"
          % ", ".join("%d x%d" % (k, n) for k, n in sorted(group_facing.items())))
    print("formations:    %s"
          % ", ".join("%s x%d" % (FORMS.get(k, k), n)
                      for k, n in sorted(forms.items())))
    grouped = {v: n for (g, v), n in order_facing.items() if g}
    loose = {v: n for (g, v), n in order_facing.items() if not g}
    print("GroupMoveOrder/MOVEORDER.facing: %s"
          % (", ".join("%d x%d" % (k, n) for k, n in sorted(grouped.items()))
             or "none — no group move order in this dump"))
    print("other MOVEORDER.facing:          %s"
          % (", ".join("%d x%d" % (k, n) for k, n in sorted(loose.items()))
             or "none"))

    print()
    if not mirrored_orders:
        print("NO group move order carries `facing 1`: the XOR term's"
              " precondition is unmet, so this capture does not reach item"
              " 23's question.")
        return 1

    frames = sorted({f for f, _ in mirrored_orders})
    print("group move orders carrying `facing 1`: %d, on %d frame(s), first"
          " %d, last %d" % (len(mirrored_orders), len(frames),
                            frames[0], frames[-1]))
    print("the XOR term's precondition is MET — a mirrored layout exists.")
    print("NOTE this is the precondition and not the event: the term fires"
          " when such an order **dies**, so the capture still needs a further"
          " re-order after the mirrored one. run31 reaches `facing 1` at 356"
          " and simply stops, which is why §13 can say the term has never"
          " fired while the flag is plainly in the file.")

    reverse_forms = [k for k in forms if k in (3, 4)]
    if reverse_forms:
        print("and a `reverse`-reading formation occurs (%s), so the mirror"
              " is visible in the slot table too"
              % ", ".join(FORMS[k] for k in reverse_forms))
    else:
        print("but no Echelon occurs, so the mirror changes no position here"
              " (§6.4: only the Echelon rows read `reverse`)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
