#!/bin/zsh
# Run one Ghidra script against the analysed riseofnations.exe, headless.
#
#   tools/ghidra/run.sh <Script.java> [script args...]
#
# Reopens the already-analysed program with -noanalysis, so a pass takes a
# couple of minutes rather than the hours the first analysis took. The script's
# own println output is filtered out of Ghidra's log and printed; anything the
# script writes to a file is wherever the script was told to write it.
#
# Environment:
#   RON_GHIDRA_PROJECT  directory holding ron.gpr (default ~/ghidra-projects)
#   GHIDRA_HOME         Ghidra install root (default: brew --prefix ghidra)
#   JAVA_HOME           a JDK 21 (default: brew's openjdk@21)
#
# Nothing this produces may enter the repo: it is derived from the user's own
# install. See docs/DECISIONS.md entries 6 and 7.
set -u
R=${RON_GHIDRA_PROJECT:-$HOME/ghidra-projects}
GH=${GHIDRA_HOME:-$(brew --prefix ghidra)}
export JAVA_HOME=${JAVA_HOME:-$(brew --prefix openjdk@21)}
export PATH="$JAVA_HOME/bin:$PATH"
HERE=${0:A:h}
SCRIPT=$1
shift
LOG=$(mktemp -t ghidra-run)
"$GH/libexec/support/analyzeHeadless" \
  "$R" ron \
  -process riseofnations.exe -noanalysis \
  -scriptPath "$HERE/scripts" \
  -postScript "$SCRIPT" "$@" > "$LOG" 2>&1
status=$?
grep -E "$SCRIPT>" "$LOG" | sed "s/^INFO  $SCRIPT> //; s/ (GhidraScript)  *\$//"
if [[ $status -ne 0 ]]; then
  echo "analyzeHeadless exited $status; full log at $LOG" >&2
  grep -E 'ERROR|Exception' "$LOG" | head -20 >&2
fi
exit $status
