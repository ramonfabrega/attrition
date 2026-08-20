#!/bin/zsh
# Decompile the whole program once, so that reading becomes grep.
#
#   tools/ghidra/export.sh [outdir]       # default $RON_GHIDRA_PROJECT/decomp
#
# Produces:
#   funcs/<Class>/<method>@<addr>.c   one file per function, 48k of them
#   types.txt                         every PDB structure, field by field
#   vtables.txt                       every vftable, slot offset -> method
#   INDEX.tsv                         addr, qualified name, file
#
# Runs the decompiler in parallel across all cores (-max-cpu); a few minutes on
# a recent machine. Also types `this` on the ~200 methods the PDB import left
# untyped, and saves that with the project, so no -readOnly.
#
# The output is a reading note, never a source: nothing in it may enter the
# repo. See docs/DECISIONS.md entry 7.
set -u
R=${RON_GHIDRA_PROJECT:-$HOME/ghidra-projects}
GH=${GHIDRA_HOME:-$(brew --prefix ghidra)}
export JAVA_HOME=${JAVA_HOME:-$(brew --prefix openjdk@21)}
export PATH="$JAVA_HOME/bin:$PATH"
HERE=${0:A:h}
OUT=${1:-$R/decomp}
mkdir -p "$OUT"
exec "$GH/libexec/support/analyzeHeadless" \
  "$R" ron \
  -process riseofnations.exe -noanalysis \
  -max-cpu "$(sysctl -n hw.ncpu)" \
  -scriptPath "$HERE/scripts" \
  -postScript ExportAll.java "$OUT"
