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

Two more checks since 2026-09-17 (the third Fable pass):

    a number the queue REFERS to — `item N`, `takes N`, `N closes`,
    `(N)` — must have been booked at some point.

Item 304 was written into the queue twice as a dependency ("wire when 304
closes", "takes 304") and never booked, and the banked suspend branch
waited on a number that did not exist. And the journal is a directory as
well as a file: `docs/journal/*.md`, one entry per landing, written by the
worker — because `docs/JOURNAL.md` conflicted on 27 of the 58 merges
since 2026-09-06 (append-only means both sides add at the same anchor).
Both texts are one journal to this guard.

And since 2026-09-28 (the eighteenth pass, parked 1006) **an item number
is up to four digits wide**. Every pattern here was `\\d{1,3}`, so from
item 1000 on `1002. ` booked nothing and `(1003)` parked nothing: the
counts stood still for a tranche and twenty-odd items were booked and
deleted by hand. The width is one constant, `N`, and
`tools/explore/test_queueledger.py` fails on a pattern that spells its
own. Five digits stay out on purpose — a frame runs to 24,000.

Run from anywhere; `tools/guard.sh` runs it between edits.
"""

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# The files an item may live in. `docs/QUEUE.md` is the only one today;
# the second is here because the sprint/backlog split lore proposed on
# 2026-09-07 would otherwise break this guard on its first commit — every
# parked item would read as a number that had left the queue in silence,
# and the guard's first act would be to fire on a hundred rows nobody
# lost. Moving a number between these files is not a deletion.
QUEUES = ["docs/QUEUE.md", "docs/PARKED.md"]
JOURNAL = ROOT / "docs/JOURNAL.md"
# One file per landing since 2026-09-17, written by the worker whose item it
# is; the single file above is the chronicle up to that day and the
# steering passes' own entries. A file's first line is the same `## date —
# item N: title` heading the single file uses, so both regexes below read
# the two texts as one.
JOURNAL_DIR = ROOT / "docs/journal"
LEDGER = ROOT / "docs/audit/queue-ledger.md"
# An item number: one to four digits, and not the head of a longer run.
N = r"\d{1,4}(?!\d)"


def git(*args):
    return subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=True
    ).stdout


def parens(text):
    """`(N)` and the slash-compound `(105/45)` — the compressed form a row
    takes once its item is finished-but-not-closed, and also the form a
    finding is *first* booked in when it rides another item's paragraph.
    Item 279 found the older `[\\d/,\\s]+` group pulling 7 and 11 out of
    "off by (11,7)" and booking both; a comma form was never used for items.
    """
    out = set()
    for group in re.findall(r"\((%s(?:/%s)*)\)" % (N, N), text):
        out |= {int(n) for n in group.split("/")}
    return out


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
        m = re.match(r"(%s)\.\s" % N, line)
        if m and (i == 0 or lines[i - 1].strip() == ""):
            out.add(int(m.group(1)))
    return out


def references(text):
    """The dependency forms an item writes about ANOTHER item — `item N`,
    `takes N`, `N closes` — the only forms that can name a number nobody
    booked, since `(N)` is itself a booking by the queue's convention.
    """
    refs = set()
    for span in re.findall(r"\b(?:items?|takes)\s+(%s)\b" % N, text, re.I):
        refs.add(int(span))
    for span in re.findall(r"\b(%s)\s+closes\b" % N, text):
        refs.add(int(span))
    return refs


def named_in(journal):
    """The numbers the journal actually **names as items**, which is not
    the same as the numbers it contains: an entry's title carries frames,
    ticks and words too, and "the word 201 → 232" would otherwise close
    item 232 by coincidence. Two constructs count, and only these:

      `item N` / `items 168, 165 and half of 170` — the phrasing the
      working agreement asks for, including the entry that closes
      several at once;
      a heading whose **title opens with the number** — "2026-09-06 —
      117 is not a lobby click", the older form, where the number is the
      subject rather than a measurement.
    """
    named = set()
    # Case-insensitive since item 279: a heading reading `Item 289` failed
    # the 289/290 merge's deletion of it.
    for span in re.findall(r"items?\s+((?:%s|,|\s|and|half of)+)" % N, journal, re.I):
        named |= {int(x) for x in re.findall(N, span)}
    for h in [l for l in journal.split("\n") if l.startswith("## ")]:
        m = re.search(r"—\s*(%s)\b" % N, h)
        if m:
            named.add(int(m.group(1)))
    return named


def main():
    ever = {}
    ever_any = set()
    live = set()
    referenced = set()
    for queue in [q for q in QUEUES if (ROOT / q).exists()]:
        for rev in reversed(git("log", "--format=%H", "--", queue).split()):
            text = git("show", "%s:%s" % (rev, queue))
            for n in booked(text):
                ever.setdefault(n, rev)
            # The paren form is too loose to say a number LEFT — the older
            # lessons list numbered itself `(4)`, and a frame count in
            # parens is not an item — but it is exactly loose enough to
            # say a number was ever WRITTEN, which is all the phantom
            # check below asks.
            ever_any |= parens(text)
        current = (ROOT / queue).read_text()
        # The working tree is a revision too: a number booked in this very
        # edit is not a phantom, and the guard runs before the commit.
        for n in booked(current):
            ever.setdefault(n, "worktree")
        ever_any |= parens(current)
        # An item is still live if a queue file books it or refers to it
        # as `(N)` — the compressed form a finished-but-not-closed row
        # takes — including the compound forms the residue rows use,
        # `(105/45)` and `(88, 72)`, where one parenthesis carries several.
        live |= booked(current)
        # `(N)` and the slash-compound `(105/45)` only: item 279 found the
        # older `[\d/,\s]+` group pulling 7 and 11 out of "off by (11,7)"
        # and booking both. A comma form was never used for items.
        live |= parens(current)
        refs = references(current)
        live |= refs
        referenced |= refs

    journal = JOURNAL.read_text()
    if JOURNAL_DIR.is_dir():
        for f in sorted(JOURNAL_DIR.glob("*.md")):
            journal += "\n" + f.read_text()
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
            r"^- \*\*(%s)\*\* (landed|dropped|unverified|not-landed|re-booked)\b" % N,
            ledger_text,
            re.M,
        )
    }
    ledgered = {n for n, d in ledger.items() if d in ("landed", "dropped", "unverified")}
    owed = sorted(n for n, d in ledger.items() if d == "not-landed" and n not in live)

    named = named_in(journal)

    silent = [
        n for n in sorted(ever) if n not in live and n not in named and n not in ledgered
    ]
    stale = sorted(n for n, d in ledger.items() if d != "re-booked" and n in live)
    phantom = sorted(n for n in referenced if n not in ever and n not in ever_any)

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

    if phantom:
        print()
        print("FAIL: the queue refers to these numbers and none was ever booked —")
        print("      a dependency on an item that does not exist (item 304, 2026-09-17):")
        for n in phantom:
            print("  %4d" % n)
        print()
        print("Book it — in docs/QUEUE.md or docs/PARKED.md — or say which item")
        print("is meant. A number is a claim that an item exists.")

    return 1 if (silent or owed or phantom) else 0


if __name__ == "__main__":
    sys.exit(main())
