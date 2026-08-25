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

$LLVM/clang --target=i686-pc-windows-msvc -ffreestanding -nostdlib -fno-builtin \
    -fno-stack-protector -fno-unwind-tables -fno-asynchronous-unwind-tables \
    -mno-sse -mno-mmx -O2 -Wall -Wextra -c $HERE/tracer.c -o $OUT/tracer.obj
# stdcall: the .def carries the @N decoration for the symbol, -k strips it from the import name
$LLVM/llvm-dlltool -m i386 -k -d $HERE/kernel32.def -l $OUT/kernel32.lib
$RLLD -flavor link /dll /machine:x86 /entry:DllMain /nodefaultlib /subsystem:windows \
    /safeseh:no /implib:$OUT/rontrace.lib /out:$INSTALL/rontrace.dll $OUT/tracer.obj $OUT/kernel32.lib
$LLVM/llvm-readobj --file-headers $INSTALL/rontrace.dll | grep -E "Machine|AddressOfEntryPoint"
$LLVM/llvm-readobj --coff-exports $INSTALL/rontrace.dll | grep -E "Name:"

python3 $HERE/funcs.py $DECOMP/INDEX.tsv $INSTALL/rontrace.funcs
python3 $HERE/patch_exe.py $INSTALL/riseofnations.exe $INSTALL/riseofnations_trace.exe rontrace.dll
