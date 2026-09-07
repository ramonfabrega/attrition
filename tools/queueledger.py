#!/usr/bin/env python3
"""The queue's own ledger: an item may not leave `docs/QUEUE.md` in silence.

`docs/QUEUE.md` **deletes** rather than strikes, and the deletion is the
claim that the item is finished — its story moved to `docs/JOURNAL.md`,
which is indexed by the item's number. That is the discipline, and it is
already the practice: of the 168 numbers retired from the queue before this
guard existed, **165 are named in the journal**.

The three that were not are why this exists. On 2026-09-06 the post-crash
rewrite that landed item 227 (`8b37e5f`, whose own message reads "Recovered
from the worktree the crash left behind") shed 196 lines of queue, and five
consecutive items went with it — 230 through 234, the only run of its kind
in the file's whole history. Two of them (230, 231) had in fact landed.
Three had not: 232's `Levels::for_player` still answers `Levels::BASE`,
233's `process_building` still keys its phase on `frame + b`, and 234's
work was found on 2026-09-07 sitting uncommitted in a reaped worktree —
406 lines of `anim.rs` that no branch carried (branch `rescue-234`).

Nothing noticed for a day, because every check anyone ran was a *branch*
check and the loss was one level below it. So the rule becomes a guard:

    a number that leaves the queue is either named in the journal
    (`item N`) or carries a line in `docs/audit/queue-ledger.md`
    saying where it went.

A number missing from both is a **failure** — that is the silent loss this
was written for. A ledger line for a number that is live in the queue again
is reported and does not fail: an item can be re-booked, and the ledger is
allowed to lag by a commit.

Run from anywhere; `tools/guard.sh` runs it between edits.
"""

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
QUEUE = "docs/QUEUE.md"
JOURNAL = ROOT / "docs/JOURNAL.md"
LEDGER = ROOT / "docs/audit/queue-ledger.md"


def git(*args):
    return subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=True
    ).stdout


def booked(text):
    """The item numbers a revision of the queue books.

    A booked item opens a paragraph — `N. **The claim**` after a blank
    line. The blank line is the discriminator that keeps prose out: the
    queue wraps its lines, so a sentence ending in a number ("the score is
    307. East Indies is unmoved...") puts a bare `307.` at the head of the
    next line and nothing else tells the two apart.
    """
    out = set()
    lines = text.split("\n")
    for i, line in enumerate(lines):
        m = re.match(r"(\d{1,3})\.\s", line)
        if m and (i == 0 or lines[i - 1].strip() == ""):
            out.add(int(m.group(1)))
    return out


def main():
    revs = git("log", "--format=%H", "--", QUEUE).split()
    ever = {}
    for rev in reversed(revs):
        for n in booked(git("show", "%s:%s" % (rev, QUEUE))):
            ever.setdefault(n, rev)

    current = (ROOT / QUEUE).read_text()
    # An item is still live if the queue books it or refers to it as `(N)`
    # — the compressed form a finished-but-not-closed row takes.
    # A live reference is `(N)`, and also the compound forms the queue's
    # residue rows use — `(105/45)`, `(88, 72)` — where one parenthesis
    # carries several numbers.
    live = booked(current)
    for group in re.findall(r"\(([\d/,\s]+)\)", current):
        live |= {int(n) for n in re.findall(r"\d{1,3}", group)}

    journal = JOURNAL.read_text()
    headings = [l for l in journal.split("\n") if l.startswith("## ")]
    # `- **N** <disposition> — …`. Three dispositions account for a number
    # and let it rest: `landed` (the work is in the tree, and the line says
    # where), `dropped` (retired on purpose, with the reason), `unverified`
    # (retired before this ledger and not audited — honest, and a debt).
    #
    # `not-landed` is a promise rather than a pardon: the work was lost, so
    # the number must be **live in the queue again**, and a `not-landed`
    # line with no queue entry behind it fails. That is what stops this
    # file from becoming the place items go to die quietly — the exact
    # failure it was written about.
    #
    # `re-booked` is the same row after the promise is kept: history, kept
    # for the reader, accounting for nothing. The item is an ordinary queue
    # entry now and owes the journal a story like any other.
    ledger_text = LEDGER.read_text() if LEDGER.exists() else ""
    ledger = {
        int(n): d
        for n, d in re.findall(
            r"^- \*\*(\d{1,3})\*\* (landed|dropped|unverified|not-landed|re-booked)\b",
            ledger_text,
            re.M,
        )
    }
    ledgered = {n for n, d in ledger.items() if d in ("landed", "dropped", "unverified")}
    owed = sorted(n for n, d in ledger.items() if d == "not-landed" and n not in live)

    # The numbers the journal actually **names as items**, which is not the
    # same as the numbers it contains: an entry's title carries frames,
    # ticks and words too, and "the word 201 → 232" would otherwise close
    # item 232 by coincidence. Two constructs count, and only these:
    #
    #   `item N` / `items 168, 165 and half of 170` — the phrasing the
    #   working agreement asks for, including the entry that closes
    #   several at once;
    #   a heading whose **title opens with the number** — "2026-09-06 —
    #   117 is not a lobby click", the older form, where the number is the
    #   subject rather than a measurement.
    named = set()
    for span in re.findall(r"items?\s+((?:\d{1,3}|,|\s|and|half of)+)", journal):
        named |= {int(x) for x in re.findall(r"\d{1,3}", span)}
    for h in headings:
        m = re.search(r"—\s*(\d{1,3})\b", h)
        if m:
            named.add(int(m.group(1)))

    silent = [
        n for n in sorted(ever) if n not in live and n not in named and n not in ledgered
    ]
    stale = sorted(n for n, d in ledger.items() if d != "re-booked" and n in live)

    print(
        "queue ledger: %d numbers ever booked, %d live, %d ledgered"
        % (len(ever), len(live), len(ledgered))
    )
    for n in stale:
        print("  note: %d is ledgered and live again — drop its ledger line" % n)

    if silent:
        print()
        print("FAIL: these numbers left docs/QUEUE.md in silence —")
        print("      not booked, not named in docs/JOURNAL.md as `item N`,")
        print("      and not in docs/audit/queue-ledger.md:")
        for n in silent:
            print("  %4d   last booked in %s" % (n, ever[n][:8]))
        print()
        print("Either the work landed (name it in the journal), or it did not")
        print("(re-book it in the queue), or it was dropped on purpose (say so")
        print("in the ledger, with the reason). A deletion is a claim.")

    if owed:
        print()
        print("FAIL: the ledger calls these `not-landed`, and the queue does")
        print("      not book them. A lost item is owed a queue entry, not a")
        print("      ledger line: re-book it, or change what the ledger says.")
        for n in owed:
            print("  %4d" % n)

    return 1 if (silent or owed) else 0


if __name__ == "__main__":
    sys.exit(main())
