#!/bin/zsh
# Build the Ghidra project: import riseofnations.exe and run full auto-analysis
# with rise.pdb resolved. Hours, once. Never re-run against an existing project;
# every other script here reopens it with -noanalysis.
#
#   tools/ghidra/analyze.sh <install>      # the directory holding riseofnations.exe
#
# The exe and the PDB are copied side by side into the project directory so
# Ghidra's universal PDB analyzer (the cross-platform one; the DIA path is
# Windows-only) finds the PDB by the name in the exe's RSDS record. The project
# lives outside the repo and outside any temp dir: Ghidra refuses a path with a
# dot-prefixed component, and a 10 MB binary's analysis is expensive enough to
# keep.
#
# Environment: as run.sh.
set -eu
INSTALL=$1
R=${RON_GHIDRA_PROJECT:-$HOME/ghidra-projects}
GH=${GHIDRA_HOME:-$(brew --prefix ghidra)}
export JAVA_HOME=${JAVA_HOME:-$(brew --prefix openjdk@21)}
export PATH="$JAVA_HOME/bin:$PATH"
mkdir -p "$R/ron-bin"
cp "$INSTALL/riseofnations.exe" "$INSTALL/sbl/rise.pdb" "$R/ron-bin/"
exec "$GH/libexec/support/analyzeHeadless" \
  "$R" ron \
  -import "$R/ron-bin/riseofnations.exe" \
  -analysisTimeoutPerFile 10800 \
  -log "$R/analyze.log"
