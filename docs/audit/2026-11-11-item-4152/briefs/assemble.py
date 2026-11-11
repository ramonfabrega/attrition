import os, re, sys
root = "/Users/rf-studio/code/fun/attrition/.claude/worktrees/att-4152"
aud = os.path.join(root, "docs/audit/2026-11-11-item-4152")
slugs = ["city-name","kills","combat-rates","victim-stamps","attrition-stamps","anti-att",
         "diplomacy-relations","diplomacy-tribute","taunts","end-of-game","lost-capital","misery",
         "marks-built","queued","odd-counters","lobby-words","spells","score","last-finished","high-buildings"]
head = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "leaders-header.md")).read().rstrip("\n") + "\n"
parts, have = [head], []
for n, s in enumerate(slugs, 1):
    p = os.path.join(aud, f"{s}-section.md")
    if not os.path.exists(p):
        continue
    t = open(p).read().strip("\n")
    m = re.match(r"## (\d+)\.", t)
    if not m or int(m.group(1)) != n:
        sys.exit(f"{s}: heading is not '## {n}.'")
    if "@00" in t:
        sys.exit(f"{s}: an @-citation")
    for sec in re.split(r"\n(?=## )", t):
        if len(sec) > 16000:
            sys.exit(f"{s}: a section of {len(sec)} chars")
    parts.append("\n" + t + "\n")
    have.append(n)
open(os.path.join(root, "docs/LEADERS.md"), "w").write("".join(parts))
print("sections:", have)
