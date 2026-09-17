#!/bin/zsh
# focus.sh — bring the original's window to the front, and say whether it is
# there at all. Thirteen scripts each had their own copy of the same
# `osascript` line, and every one of them named the process
# `riseofnations.exe` — which is what **CrossOver** called it. Under free
# WineHQ every Wine GUI process is called `wine` to System Events, whatever
# executable it is running, so the window has to be found by its **title**.
#
#   zsh tools/gamelog/focus.sh            # focus it, exit 0 if found
#   zsh tools/gamelog/focus.sh --title    # print the title, or nothing
#
# `RON_WINDOW` overrides the title match (the game's is "Rise of Nations:
# Extended Edition" for both the stock and the traced executable, since the
# title comes from the window class rather than the file name).
#
# Note the Automation grant: run this under `viadriver.sh`, or the query
# comes back "not allowed assistive access" and a caller that reads an empty
# answer as "no window yet" waits forever.

title=${RON_WINDOW:-Rise of Nations}
mode=${1:-focus}

found=$(osascript - "$title" "$mode" <<'EOF'
on run argv
  set want to item 1 of argv
  set mode to item 2 of argv
  tell application "System Events"
    repeat with p in (every process whose name is "wine")
      repeat with w in (every window of p)
        set n to name of w
        if n contains want then
          if mode is not "--title" then set frontmost of p to true
          return n
        end if
      end repeat
    end repeat
  end tell
  return ""
end run
EOF
) || exit $?
if [[ -z "$found" && "$mode" != "--title" ]]; then
  print -u2 -- "focus: no matching game window: $title"
  exit 1
fi
print -r -- "$found"
