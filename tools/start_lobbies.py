#!/usr/bin/env python3
"""The lobbies on disk with a `[Start Game]` dump (item 4033, `docs/START.md` §5).

The start track's counter is `Start: <built> of <N> lobbies`; N is this
scan. A capture counts when its `GAME` block carries start records ahead of
its first `FRAME` — `GameLog::begin_game`'s dump: a `WORLD`, `UNITDATA`,
`BUILDDATA` or a `FULL DUMP` child before any frame. A windowed capture
opens on a frame and carries none. Its lobby is the `GAMEINFO` block's
text, the key `rondata::diff::setup::same_lobby` compares (every field
equal); two captures of one lobby are one lobby.

Usage: start_lobbies.py [LOGDIR] [--list]

Reads each file only to its first frame. Prints `lobbies <N> captures <M>`.
"""
import hashlib
import os
import sys

DEFAULT = os.path.expanduser(
    "~/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"
)
START_KIDS = ("BEGIN WORLD", "BEGIN UNITDATA", "BEGIN BUILDDATA", "BEGIN FULL DUMP")


def head(path):
    """(gameinfo text, has start records) read to the first frame."""
    info, in_info, depth, in_game = [], False, None, False
    with open(path, "rb") as f:
        for raw in f:
            line = raw.decode("latin-1").rstrip("\r\n")
            body = line.lstrip(" ")
            indent = len(line) - len(body)
            if body.startswith("BEGIN FRAME"):
                return info, False
            if in_info:
                if indent <= depth and body.startswith("BEGIN ") and body != "BEGIN PLAYER":
                    in_info = False
                elif indent <= depth - 1:
                    in_info = False
                else:
                    info.append(body)
                    continue
            if body == "BEGIN GAMEINFO":
                in_info, depth = True, indent
                continue
            if line == "BEGIN GAME":
                in_game = True
                continue
            if in_game and indent >= 1 and body in START_KIDS:
                return info, True
    return info, False


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    listing = "--list" in sys.argv
    logdir = args[0] if args else DEFAULT
    lobbies = {}
    for name in sorted(os.listdir(logdir)):
        if not (name.startswith("gamelog-") and name.endswith(".txt")):
            continue
        info, start = head(os.path.join(logdir, name))
        if not start or not info:
            continue
        key = hashlib.sha1("\n".join(info).encode()).hexdigest()[:12]
        lobbies.setdefault(key, []).append(name)
    captures = sum(len(v) for v in lobbies.values())
    if listing:
        for k, v in sorted(lobbies.items(), key=lambda kv: kv[1][0]):
            print(k, len(v), v[0])
    print(f"lobbies {len(lobbies)} captures {captures}")


if __name__ == "__main__":
    main()
