#!/bin/zsh
# wow64bop.sh [outdir] — build wow64bop.exe, the standalone falsifier for
# docs/ORACLE.md's "226: the fault is the bop, not the handler".
#
# Same toolchain as build.sh and nothing else: Homebrew LLVM's clang and
# llvm-dlltool, plus the pinned Rust toolchain's rust-lld in link mode. The
# product is a ~3 KB 32-bit console PE that depends on kernel32 alone — no
# install, no graphics, no display, no game. Copy it to any host with Wine
# and run it; the verdict is on stdout, and the source's header says how to
# read it.
#
#   zsh tools/trace/wow64bop.sh
#   wine ${TMPDIR:-/tmp}/rontrace-build/wow64bop.exe
set -e
HERE=${0:A:h}
LLVM=${LLVM:-/opt/homebrew/opt/llvm/bin}
HOST=$(rustc -vV | sed -n 's/^host: //p')
RLLD=$(rustc --print sysroot)/lib/rustlib/$HOST/bin/rust-lld
OUT=${1:-${TMPDIR:-/tmp}/rontrace-build}
mkdir -p $OUT

$LLVM/clang @$HERE/compile_flags.txt -O1 -c $HERE/wow64bop.c -o $OUT/wow64bop.obj
$LLVM/llvm-dlltool -m i386 -k -d $HERE/wow64bop.def -l $OUT/wow64bop-k32.lib
$RLLD -flavor link /machine:x86 /entry:start /nodefaultlib /subsystem:console \
    /safeseh:no /out:$OUT/wow64bop.exe $OUT/wow64bop.obj $OUT/wow64bop-k32.lib
ls -la $OUT/wow64bop.exe
