#!/bin/zsh
# build.sh <install> [<decomp export dir>]
#
# Builds rontrace.dll from tracer.c (freestanding, 32-bit, no CRT) with the
# tools already on this machine — Homebrew LLVM's clang and llvm-dlltool and
# the pinned Rust toolchain's rust-lld in link mode — and stages everything
# the trace needs into the install directory (gitignored; nothing here enters
# the repo):
#
#   <install>/rontrace.dll               the instrument
#   <install>/rontrace.funcs             every function entry, u32 RVAs
#   <install>/riseofnations_trace.exe    a copy of the exe that imports it
#
# Then write <install>/rontrace.cfg (`window=0-3`) and launch the copy the
# way the original is launched (docs/ORACLE.md). The log lands beside it as
# rontrace.log; read it with report.py.
set -e
HERE=${0:A:h}
INSTALL=${1:?install dir}
DECOMP=${2:-$HOME/ghidra-projects/decomp}
LLVM=${LLVM:-/opt/homebrew/opt/llvm/bin}
HOST=$(rustc -vV | sed -n 's/^host: //p')
RLLD=$(rustc --print sysroot)/lib/rustlib/$HOST/bin/rust-lld
OUT=${TMPDIR:-/tmp}/rontrace-build
mkdir -p $OUT

# Compile flags live in compile_flags.txt, one per line: clang expands it as a
# response file, and clangd reads the same file, so editors analyze this file as
# the freestanding 32-bit Windows target it is rather than as host macOS code.
# TRACER_DEFS: extra -D flags for a diagnostic build (`-DFLUSH_RECS=1` writes
# every record as it is made, so a death loses nothing; slow).
$LLVM/clang @$HERE/compile_flags.txt -O2 ${=TRACER_DEFS} -c $HERE/tracer.c -o $OUT/tracer.obj
# stdcall: the .def carries the @N decoration for the symbol, -k strips it from the import name
$LLVM/llvm-dlltool -m i386 -k -d $HERE/kernel32.def -l $OUT/kernel32.lib
$RLLD -flavor link /dll /machine:x86 /entry:DllMain /nodefaultlib /subsystem:windows \
    /safeseh:no /implib:$OUT/rontrace.lib /out:$INSTALL/rontrace.dll $OUT/tracer.obj $OUT/kernel32.lib
$LLVM/llvm-readobj --file-headers $INSTALL/rontrace.dll | grep -E "Machine|AddressOfEntryPoint"
$LLVM/llvm-readobj --coff-exports $INSTALL/rontrace.dll | grep -E "Name:"

# The coverage table needs the executable's own bytes (the displaced
# prologues) and capstone, which `uv run` supplies from the script's header.
uv run $HERE/funcs.py $DECOMP/INDEX.tsv $INSTALL/riseofnations.exe $INSTALL/rontrace.funcs
python3 $HERE/patch_exe.py $INSTALL/riseofnations.exe $INSTALL/riseofnations_trace.exe rontrace.dll
