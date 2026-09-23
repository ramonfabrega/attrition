# Prior art: static recompilation of a PE32 x86 game to arm64, and in-process x87 oracles

Survey date 2026-09-22. Dates are the last commit on the default branch (or `pushed_at`), read with `gh api repos/<r>` that day.
"Verified" means read in the project's README or source. Claims that rest on search snippets or memory are marked **unverified**.

Spike context: Rise of Nations `riseofnations.exe` (PE32, MSVC, 2003, full private PDB giving function bounds, types and vtables).
Goal: run one original function at a time as native arm64 code on macOS, in-process beside Rust, with floats bit-exact to the original (x87 80-bit plus the precision-control word, and possibly SSE).

## 1. N64Recomp (MIPS -> C) and Zelda64Recomp (Majora's Mask)

- Repo: https://github.com/N64Recomp/N64Recomp. MIT, last commit 2026-05-27, about 8.1k stars, active.
- Port: https://github.com/Zelda64Recomp/Zelda64Recomp. GPL-3.0, last commit 2026-05-17. It ships Apple Silicon macOS and ARM64 Linux builds: "A Mac with Apple Silicon ... ARM64 builds will work on any ARM64 CPU" (README).
- Translates: MIPS (VR4300) into C. The translation is "very literal": one C function per original function, CPU state in a `ctx` struct, memory through an `rdram` base (README, https://github.com/N64Recomp/N64Recomp#how-it-works).
- (a) Function discovery: **not** done by analysis. "Currently, the only way to provide the required metadata is by passing an elf file". In practice that is the decomp project's ELF with symbols. Configuration is a TOML file (stubs, skips, single-instruction patches). (README "How to Use")
- (b) Indirect calls: `jalr` becomes `LOOKUP_FUNC(ctx->r25)(rdram, ctx)`. The runtime keeps a table of loaded sections and maps an address to its recompiled function (README "Overlays"). `jal` becomes a direct C call, and identified tail-calls become calls.
- (c) Jump tables: "attempting to turn a `jr` instruction into a switch-case statement if it can tell that it's being used with a jump table". This is pattern-based and tuned for IDO and gcc 2.7.2 output: "Modern mips gcc may trip up the recompiler" (README).
- (d) Exceptions: not applicable (no C++ EH on N64 targets) and not addressed.
- (e) Float: the MIPS FPU is IEEE single/double and maps onto host `float`/`double`. The FCSR rounding mode maps to `fesetround` (`set_cop1_cs`, `include/recomp.h:288-306`). Functions compile under `STDC FENV_ACCESS ON` (clang) or `optimize("rounding-math")` (gcc), so constant folding respects the environment (`include/recomp.h:20-33`). NaN is only `assert`-checked (`NAN_CHECK`, `recomp.h:396`). This ISA has no extended-precision problem, so nothing here carries over to x87.
- (f) OS calls: none are translated. The runtime (N64ModernRuntime, https://github.com/N64Recomp/N64ModernRuntime) reimplements libultra entry points by symbol replacement.
- (g) Self-modifying code / data in code: overlays are handled through relocation macros (`RELOC_HI16`) that the runtime patches. There is no general self-modifying-code support.
- It also ships `LiveRecomp`, a sljit-based JIT of the same IR for mods (`LiveRecomp/live_generator.cpp` in the source tree).
- (i) Build on for PE32 x86? **Not as code.** The decoder is rabbitizer (MIPS only) and the ISA semantics are MIPS-specific. x86's variable-length decoding, flags and x87 stack are the hard parts, and N64Recomp never faced them. **Yes as architecture.** It gets its function list from symbols (we have the PDB). It emits one C function per original function with a state struct. It replaces individual functions through weak symbols and link order ("single file output mode", README). It resolves indirect calls through `LOOKUP_FUNC`. This is exactly the shape the spike wants.

## 2. XenonRecomp (PPC Xbox 360 -> C++) and UnleashedRecomp (Sonic Unleashed)

- Repo: https://github.com/hedge-dev/XenonRecomp. MIT, last commit 2025-08-04, so quiet for 13 months. Whether forks continue the work is **unverified**.
- Port: https://github.com/hedge-dev/UnleashedRecomp. GPL-3.0, last commit 2026-06-29. The README claims "Windows and Linux support" and requires AVX on x86-64. It claims no macOS release.
- Translates: PowerPC (Xenon) plus VMX128 into C++, compiled with clang. It uses a state struct plus a 32-bit guest base pointer. Loads and stores are byte-swapped and marked `volatile` "to prevent Clang from doing unsafe code reordering" (README "Instructions").
- (a) Function discovery: XenonAnalyse uses `.pdata` plus `bl` targets. It "struggles with functions containing jump tables, since they look like tail calls". The workaround is manual `functions` boundaries in TOML (README).
- (b) Indirect calls / vtables: a runtime "perfect hash table". Function pointers are stored just past the guest image in the base region and indexed by original address (README "Indirect Functions").
- (c) Jump tables: a pattern search for `mtctr r0`/`bctr` emits a TOML of tables, which become real `switch` statements. "there is no fully generic solution for handling jump tables" (README "Jump Tables").
- (d) Exceptions: "The recompiler currently does not support exceptions." `setjmp`/`longjmp` are redirected to native implementations. EH data between functions is skipped through `invalid_instructions` (README).
- (e) Float/VMX: the FPU keeps denormals and VMX flushes them. The current mode is tracked in the state struct and toggled before each instruction. On arm64 it writes FPCR bits 19 and 24 directly (`mrs/msr fpcr`, `XenonUtils/ppc_context.h:241-255`); on x86 it writes MXCSR through SIMDe. VMX runs on x86 intrinsics, and on arm64 through SIMDe (README). **Not bit-exact by construction**: `fmadd` is emitted as `a*b+c` (`XenonRecomp/recompiler.cpp:923-926`), so whether it fuses depends on the host compiler's `-ffp-contract`. `fmadds` is emitted as `double(float(a*b+c))`, which rounds twice where the hardware rounds once. I saw no model of FPSCR sticky flags or exceptions.
- (f) OS calls: the recompiler "does not provide a runtime implementation"; imports are the port's job (README disclaimer). UnleashedRecomp reimplements the kernel/XAM surface by hand (**unverified** in detail).
- (g) Self-modifying code: not supported. MMIO is "currently unimplemented".
- Hooks: every PPC function is weakly aliased, so a native replacement can still call the original. There are also "mid-asm hooks" at arbitrary instruction addresses, with register arguments, return and jump control (README). This is the closest existing design to "original function beside a reimplementation".
- (i) Build on for PE32 x86? **Not as code** (PPC decoder, big-endian, fixed-width ISA). **Yes as design**: an external source for function boundaries (the PDB is strictly better than `.pdata`), a jump-table TOML, the weak-alias hook model, and the arm64 FPCR handling pattern.

## 3. Static translators for 32-bit x86 (PE and otherwise)

### 3a. SR / SRW (M-HT, Roman Pauer): DOS and Win32 x86 -> x86 / x64 / LLVM IR ("llasm")
- Repo: https://github.com/M-HT/SR. MIT (plus a GPLv2+/LGPLv2.1+ dual release, README). Last commit 2026-08-27, 419 stars. Maintained for years.
- It is the only mature project found that statically recompiles **Win32 PE** games and ships **macOS arm64** builds: "create Windows (x86/x64) or Linux (x86/x64/arm/arm64/riscv64) or MacOS (x64/arm64) versions" (README). The Win32 targets are Septerra Core (`Septerra104.exe`, 729 KB) and Battle Isle 3 (an EXE plus DLLs) (`SRW-games/README.md`).
- Pipeline: `SRW` (for Win32) disassembles with udis86 1.7.2. It emits NASM x86/x64 or `.llasm`, and `llasm` turns `.llasm` into LLVM IR (`SRW/README.md`, `llasm/README.md`). arm64 goes through the llasm/LLVM path.
- (a) Discovery: whole-program traversal, driven by per-game hint files: a `relocations.csv` of `(site, target)` pairs (21,588 rows for Septerra), a `bssborder.csv` and an `SR.cfg` (`SRW-games/Septerra Core/SRW/`). The relocation list is what separates pointers from integers, so data keeps its original addresses. **Unverified**: how SRW gets relocations for an EXE linked without `.reloc`. MSVC EXEs usually ship without them, so `riseofnations.exe` would need checking.
- (b) Indirect calls / vtables: code pointers in data are known from the relocations and remapped to recompiled labels. This is inferred from the design, not read in the dispatch source (**unverified in detail**).
- (e) Float: the llasm output sets `EMULATE_FPU 1` (`SRW/SR_defs.h:48-50`). The x87 is emulated as **`double _st[8]`** (`SR/llasm-support/llasm_cpu.h:32`), with helpers such as `x87_fadd_double` and switches on rounding control (`SR/llasm-support/llasm_float.c:175,360-474`). Rounding control is honoured, but values are **64-bit, not 80-bit**, so precision control is moot. It is not bit-exact for code whose results depend on 80-bit intermediates.
- (f) Imports: per-game C reimplementations of the Win32 surface in `games/` (SDL-based).
- (i) Build on? **This is the closest shipping precedent for PE32 -> arm64 macOS.** But it is whole-program, not one function at a time beside foreign code. It is assembler-level (register globals), game-specific in its hint files, and 53-bit on x87. It is worth reading for the relocation and data-layout approach. Forking it for per-function interop would amount to a rewrite.

### 3b. pcrecomp / xboxrecomp / rol (sp00nznet): x86-32 PE -> C
- https://github.com/sp00nznet/pcrecomp: the LICENSE file says MIT (GitHub shows NOASSERTION). Created 2026-02-27, last push 2026-09-22, 32 stars. Python plus capstone.
- https://github.com/sp00nznet/xboxrecomp: MIT, created 2026-03-06, last push 2026-09-20. The same family of x86-32 lifters, applied to XBE.
- **https://github.com/sp00nznet/rol is a static recompilation of *Rise of Nations: Rise of Legends*** (BHG's rts2 engine, MSVC 7.1, 13.25 MB `.text`), created 2026-09-10. It is at "Phase 3 (seed)", meaning discovery and scoring only; lifting is "Pending". It has no PDB: "no source for Rise of Legends ... IDA is a strong second opinion, not truth". Discovery scored **99.3% recall / 44.9% precision** against IDA, and reached 1.00x duplication after an ownership fix (rol README, "The scorecard" and "The fix, measured").
- Translates: x86-32 into "readable C", one C statement per instruction, relocation-aware. There are two models: global registers (`lift32.py`) and a reentrant CPU struct (`lift32_cpu.py`), which "hybrid builds" require (pcrecomp README).
- (a) Discovery: `disasm32.py` does recursive descent plus scans of the data sections for pointers. `score_recovery.py` scores the result against IDA, a linker map or a PDB: "A reference is only ground truth if it came from symbols" (README).
- (b) Indirect calls: "Three-Tier Dispatch", meaning manual overrides, then an auto-generated binary-search table over every lifted function address, then import bridges (`docs/PHILOSOPHY.md` section 5). `runtime/hybrid/` routes vtables so that real (unlifted) code can call lifted functions and the reverse (`docs/HYBRID.md`). **That is the "original function beside a reimplementation" mode, run the other way round** (lifted code beside real x86).
- (c) Jump tables: `jmp [reg*4+table]` is recognised and the table is read as consecutive dwords (`tools/lift/lift32_cpu.py:237-254,696,1608`).
- (d) SEH: `fs:` accesses go to `__readfsdword`/`__writefsdword`, "so SEH prologues work" (`lift32_cpu.py:27,353`). That uses the **real** Win32 TIB, so it only works on a Windows x86 host. MSVC's SEH/EH funclets are a documented source of crashes in hybrid builds (`docs/HYBRID.md` "Rule 1").
- (e) Float: **not bit-exact.**
  - "The x87 stack here is modelled as doubles" (`runtime/recomp32_cpu/cpu.h:564-575`).
  - `fldcw` is "ignored", and `fnstcw` returns the constant `0x027F` (`lift32_cpu.py:981-984`).
  - "neither rounding modes nor FP exceptions are modelled" (`lift32_cpu.py:762-767`).
  - SSE scalar single is deliberately kept as `float` (`cpu.h:30-40`).
  - The xboxrecomp lifter does store `fldcw` into `g_fp_control_word`, but its stack is still `double g_fp_stack[8]` (xboxrecomp `tools/recomp/lifter.py:10,3473`). Its conformance harness "deliberately runs the x87 at 53-bit precision" (xboxrecomp README).
- (f) Imports: SDL2/Win32 compatibility shims (`runtime/compat`). The hybrid mode keeps the real CRT and MFC.
- Verification: `tools/lift/difftest.py` runs the lifted C against **Unicorn** and diffs every register, flag and byte (README, "Did the lifter get the semantics right?").
- Target: the build template sets "32-bit target (`-A Win32`)", no ASLR, fixed addresses (`docs/PIPELINE.md:369-374`). Guest pointers are host pointers (`(void*)(uintptr_t)a` in `cpu.h`). On arm64 macOS the low 4 GB sits inside `__PAGEZERO`, and xboxrecomp hit exactly that: "fatal on arm64 macOS, where every fixed base sits inside `__PAGEZERO`" (xboxrecomp README).
- (i) Build on? The **tooling ideas** transfer directly: scoring discovery against the PDB, a Unicorn difftest per lifted function, hybrid vtable routing. The **lifter does not**: its x87 model is double and its SEH relies on the host TIB. This is a very young, fast-moving project (the rol repo carries a `CLAUDE.md`). Watch rol, because it is a sibling BHG engine. If it reaches Phase 4, its per-function findings on MSVC 7.1 output will transfer.

### 3c. remill + McSema (Trail of Bits): x86/amd64/AArch64/SPARC -> LLVM bitcode
- remill: https://github.com/lifting-bits/remill. Apache-2.0, last commit 2026-08-27. McSema: https://github.com/lifting-bits/mcsema, **archived**, last push 2022-04-26, AGPL-3.0.
- McSema lifted "both Linux (ELF) and Windows (PE) executables ... x86 and amd64 ... including integer, X87, MMX, SSE and AVX". It recovered the CFG through **IDA Pro** (`mcsema-disass`) and claimed C++ exception support (McSema README and its comparison table). The successor path is Anvill, which is decompilation-oriented.
- (e) remill's x87: "On non-x86 architectures, native_float80_t is defined as a double" (`include/remill/Arch/Runtime/Operators.h:638`, `lib/Arch/X86/Semantics/X87.cpp:131`), so an arm64 host gets **53-bit x87**. `FLDCW` stores the word, then forces `cw.pc = kPrecisionSingle` and applies only the rounding field through `__remill_fpu_set_rounding` (`X87.cpp:1337-1343`). **Precision control is ignored.** BinRec's authors hit "McSema's handling of double-precision floating point operations in 32-bit applications" (BinRec paper, section 6).
- (b)/(c): remill lifts instructions only. Recovering indirect flow is the front end's job (for McSema, IDA's xrefs and switch tables).
- (i) Build on? remill is maintained and credible as an instruction-semantics library into LLVM IR. But its runtime model (`__remill_*` memory and state intrinsics) is heavy, and its x87 on arm64 is double. Nothing PE-aware in this family is still maintained (McSema is archived).

### 3d. rev.ng: lifter and decompiler built on QEMU TCG
- https://github.com/revng/revng. GPL-2.0 as a whole, individual files MIT (README). Last commit 2026-09-22, very active.
- Architectures include `x86` (i386), x86_64, arm, aarch64, mips/mipsel and systemz (`include/revng/Model/Architecture.h`). It has a `PECOFFImporter.cpp` and imports PDBs ("Windows debug information (PDB)", `share/doc/revng/what-is-revng.md`).
- It is now a **decompiler**: "The C code we emit is designed to be valid and recompilable" (what-is-revng.md). It lifts through QEMU TCG ("we never emulate any code, we simply use QEMU as a lifter"), and inlines QEMU's `fpu/` (softfloat) helpers as a runtime library (`lib/HelperInliningAnalyses/MarkInlineHelpersUpTo.cpp:37`). Its x87 semantics are therefore QEMU's floatx80. Section 4 covers whether that honours precision control.
- The old `revng translate` mode (binary to binary) does not appear in the current CLI docs (`share/doc/revng/references/cli/`); whether it still exists is **unverified**. rev.ng "can only run on Linux x86-64" (`user-manual/initial-setup.md`).
- (i) Build on? Its decompiled C is written for humans; it is not a bit-exact, per-function recompilation path. It could serve as a second reader, not as the recompiler.

### 3e. RetDec (Avast): decompiler
- https://github.com/avast/retdec. MIT, last commit 2026-05-26. "currently in a **limited maintenance mode**" (README). It supports PE and 32-bit x86 (README). It is a decompiler, not a recompiler, and it makes no claim of bit-exactness. Not a base.

### 3f. BinRec and Polynima (research recompilers)
- BinRec (EuroSys 2020), https://github.com/securesystemslab/BinRec, last push 2022-11-29. It lifts **dynamic traces** to LLVM through S2E/QEMU and recompiles them, with "control flow miss handlers" that fall back into the original code. The prototype "targets single threaded 32-bit x86 binaries on Linux". S2E "emulates floating-point instructions using integer instructions", and the authors limited their evaluation "to the CINT subset of SPEC" (paper sections 4-5, https://download.vusec.net/papers/binrec_eurosys20.pdf). Its binary stitching is ELF-only.
- Polynima (EuroSys 2024, https://doi.org/10.1145/3627703.3650065) does hybrid static and dynamic recompilation of "x86/x64 multithreaded binaries" at a 1.23x slowdown (abstract). I found no public source repository (**unverified**).
- (i) Neither is PE-aware, and neither addresses x87 fidelity.

### 3g. Instrew
- https://github.com/aengelke/instrew. LGPL-2.1, last push 2026-09-10. An LLVM-based **dynamic** translator. Its guests are "x86-64, AArch64, and RISC-V64" and its hosts x86-64 and AArch64 (README). It has **no i386 guest** and runs Linux user-mode only. Not relevant.

### 3h. reccmp / isledecomp (LEGO Island): verification, not translation
- https://github.com/isledecomp/reccmp. AGPL-3.0, last push 2026-09-22.
- It "compares an original binary with a recompiled binary, provided a PDB". Functions are matched through annotations such as `// FUNCTION: LEGO1 0x100b12c0`. It verifies "functions, virtual tables, variable offsets" and reports an asm-level match percentage.
- It supports "C++ compiled to 32-bit x86 with old versions of MSVC (like 4.20)"; newer MSVC versions are in progress (README). The host project is https://github.com/isledecomp/isle (LGPL-3.0).
- This is the **matching-decomp** alternative: rewrite a function in C++, build it with the *original* MSVC (7.x for RoN), and diff the assembly. That gets bit-exact floats for free, because the compiler and its x87 codegen are the same. The result runs only on x86 and needs the period compiler.

### 3i. OpenRCT2's original hook-and-replace
- Early OpenRCT2 was a DLL. The old README (mirrored at https://github.com/kevinburke/OpenRCT2) says:
  - "The original executable rct2.exe has been patched so that openrct2.dll and WinMain are in the DLL import table ... the DLL can then run all the decompiled code whilst still being able to read / write to the rct2.exe memory model and run rct2.exe procedures."
  - "Until all procedures of the original game are re-written in C, the project must remain a DLL".
- `RCT2_CALLPROC*` were asm thunks into the exe; the `EBPSAFE` variants saved `ebp` (https://github.com/OpenRCT2/OpenRCT2/wiki/Decompiling-Tips-OllyDbg; that detail comes from search snippets and is **unverified**).
- This is the in-process oracle pattern with no translation at all. It needs an x86 host, which on this machine means Wine (see section 4, Rosetta).

### 3j. x86port (SomeoneIsWorking): a three-week-old x86-32 JIT that dropped static recompilation
- https://github.com/SomeoneIsWorking/x86port. **No license file**, created 2026-09-01, 0 stars.
- It describes itself as "replacing static recompilation": a JIT with x64 and ARM64 backends and a Zydis decoder. It has "Software x87 functions: host-independent ext80 arithmetic" through a pinned softfloat (`docs/codemap.md:38`, `cmake/softfloat.cmake`).
- But its ARM64 JIT path "retains binary64 x87 state without claiming extended precision" (`docs/project-state.md:69`).
- Not usable (no license, not qualified). It is noted as a signal: ext80 softfloat is the known answer, and this project has not yet wired it in.

## 4. Dynamic translators as the in-process oracle

### 4a. Unicorn (QEMU TCG as a library)
- https://github.com/unicorn-engine/unicorn. GPL-2.0. The default branch `master` was last committed 2026-02-17; `dev` on 2026-08-28. It is built on **QEMU 5.0.1** (`qemu/VERSION`, the same on both branches). The official Rust binding is `bindings/rust`, crate `unicorn-engine` 2.1.5, GPL-2.0 (`bindings/rust/Cargo.toml`). This repo already drives it from `tools/emu/callfn.py` on this machine, so it is proven to run on arm64 macOS here.
- **x87 precision control IS honoured for the basic operations.** `update_fp_status` maps FPUC bits 8-9 to `set_floatx80_rounding_precision(32|64|80)` and RC to the softfloat rounding mode (`qemu/target/i386/fpu_helper.c:590-619`; `fldcw` goes through `cpu_set_fpuc`). `roundAndPackFloatx80` rounds the significand to 24 or 53 bits but keeps the 15-bit exponent (`qemu/fpu/softfloat.c:3921-3960`), and that is the real x87's PC semantics. So add, sub, mul, div, sqrt, the loads and stores in m32/m64/m80, and the integer conversions are softfloat floatx80 and should be bit-exact to hardware under any PC/RC. **Inferred from the source, not yet run against hardware.**
- **Transcendentals are NOT exact in Unicorn.** `fsin`, `fcos`, `fptan`, `fpatan`, `f2xm1` and `fyl2x` convert ST0 to host `double`, call libm, and convert back (`fpu_helper.c:716-770,966-980`). Upstream QEMU has since reimplemented `f2xm1`, `fpatan` and `fyl2x` in floatx80 (`target/i386/tcg/fpu_helper.c:1148,1381,2171` on qemu master, file last touched 2026-09-11). `fsin`, `fcos` and `fptan` are still host double there too (`:2402,1309`).
- SSE: a separate `sse_status` softfloat, with MXCSR DAZ/FZ honoured (`fpu_helper.c:1603-1607`). IEEE single/double softfloat is exact by construction.
- Per-call overhead: **no published measurement found.** It is dominated by `uc_emu_start` setup plus TB translation on first entry. `UC_CTL_TB_REQUEST_CACHE`/`UC_CTL_TB_FLUSH` exist to manage the translation cache (`include/unicorn/unicorn.h:584-592`). This needs measuring on this machine; the spike should time a warm call.

### 4b. FEX-Emu
- https://github.com/FEX-Emu/FEX. MIT, last commit 2026-09-22, very active. "run x86 applications on ARM64 **Linux** devices ... both 32-bit and 64-bit binaries ... alongside Wine/Proton" (README). No macOS host.
- x87 is **full 80-bit by default**: Berkeley SoftFloat-3e `extF80` (`FEXCore/Source/Common/SoftFloat.h:17-18`). Precision control and rounding control are **honoured**: `SoftFloatStateFromFCW` maps PC to `roundingPrecision = 32/64/80` and RC to the softfloat mode (`FEXCore/Source/Interface/Core/Interpreter/Fallbacks/F80Fallbacks.h:12-35`). Transcendentals use 128-bit cephes (`sinl`, `tanl`, `atan2l`, `exp2l`, `log2l` in `SoftFloat.h:350-501`). They are accurate, but that says nothing about matching Intel's microcode bit for bit (**unverified**).
- `X87ReducedPrecision` (default `false`): "Emulates X87 floating point using 64-bit precision. This reduces emulation accuracy" (`FEXCore/Source/Interface/Config/Config.json.in:629-635`).
- (i) As an oracle: it has the best x87 fidelity of the translators here, but it is Linux-only. Embedding FEXCore in a macOS process is **unverified and likely heavy**. Its `F80Fallbacks`/`SoftFloat.h` code is MIT and liftable as an **x87 reference library** in its own right.

### 4c. Box86 / Box64
- https://github.com/ptitSeb/box86 (MIT, last push 2026-09-11) and https://github.com/ptitSeb/box64 (MIT, last push 2026-09-22). Linux userspace ("Linux Userspace x86-64 Emulator", box64 README).
- x87 runs on **host hardware floats**: "box uses hardware float (with some tricks to keep 80bits when needed, like on some data copy used by old games)" (https://box86.org/2022/03/box86-box64-vs-qemu-vs-fex-vs-rosetta2/).
- `BOX64_DYNAREC_X87DOUBLE`: "0: Try to use float when possible [Default] 1: Only use Double 2: Check Precision Control low precision". `BOX64_X87_NO80BITS`: "0: Try to handle 80bits long double as precise as possible [Default] 1: Use 64bits double" (box64 `docs/USAGE.md:337-343,549-554`).
- Not bit-exact at PC=64, and even the default may use *float*. Unsuitable as an oracle.

### 4d. Rosetta 2
- "Rosetta 2 has a full, slow, software implementation of x87's 80-bit floating point numbers", and "Rosetta 2 also apparently supports the full 32-bit instruction set for Wine", which the author himself flags as "(I haven't investigated this myself.)" (Dougall Johnson, https://dougallj.wordpress.com/2022/11/09/why-is-rosetta-2-fast/).
- Whether Rosetta honours **precision control**, and whether its transcendentals match Intel's, is **unverified**; Apple publishes nothing. rosettax87 swaps in "less precise but significantly faster x87 instruction handlers" (https://github.com/Lifeisawful/rosettax87, archived; continued at https://github.com/WineAndAqua/rosettax87). x87sidecar (https://github.com/rdbell/x87sidecar, MIT, 2026-09-14) is an in-process JIT replacing Rosetta's x87 handlers. It is tested against "self-checking binaries under stock Rosetta", and notes "at the 53-bit precision Windows processes run at the unfused form is the exact one" (README).
- **Critical for the spike: this project's oracle already runs under Rosetta.** RoN runs in Wine on Apple Silicon, with its 32-bit code under "Rosetta's 32-bit translation" (`docs/ORACLE.md:2352-2356`), and "Rosetta 2 ends with macOS 28, autumn 2027" (`docs/ORACLE.md:2256`). So every per-frame dump the diff trusts was computed by **Rosetta's x87**, not an Intel FPU. "Bit-exact to the original" in practice means bit-exact to Rosetta's software x87, unless a dump is re-taken on real x86. If Rosetta is 80-bit and honours PC, softfloat (QEMU/FEX/Berkeley) matches it on + - * / sqrt, and transcendentals are the residual risk.
- Rosetta cannot be called in-process from a native arm64 process (it translates whole x86 processes). The in-process route to Rosetta's semantics is the OpenRCT2 pattern inside the Wine process, which is not "beside Rust on arm64".

### 4e. Blink (jart)
- https://github.com/jart/blink. ISC, last push 2025-12-10. Emulates x86-64 Linux (with 16/32-bit modes) and runs on "macOS (x86, ARM)". The JIT is "for programs running in long mode (64-bit)" (README).
- Its x87 is not 80-bit on these hosts: tests were excluded because "it wanted x87 long double to have 80-bit precision" (README "Testing"). Not an oracle for x87.

## Summary table

| Project | Translates | Discovery | Indirect/vtable | Jump tables | EH | x87 fidelity | PE32? | arm64 mac? | License | Last commit | Build on? |
|---|---|---|---|---|---|---|---|---|---|---|---|
| N64Recomp | MIPS->C | symbols (ELF) | `LOOKUP_FUNC` table | pattern -> switch | n/a | n/a (IEEE, fesetround) | no | yes (Zelda64Recomp) | MIT | 2026-05-27 | design only |
| XenonRecomp | PPC->C++ | .pdata + bl + TOML | hash table past image | `mtctr` pattern TOML | none (setjmp native) | n/a; fmadd unfused | no | via SIMDe/FPCR (code) | MIT | 2025-08-04 | design only |
| SR/SRW | x86 DOS/Win32 -> asm/LLVM IR | whole-program + relocations.csv | relocation remap | yes (whole-program) | unverified | double, RC honoured | **yes** | **yes (ships)** | MIT/GPL/LGPL | 2026-08-27 | read, not fork |
| pcrecomp/xboxrecomp/rol | x86-32 -> C | recursive descent + data scan, scored vs IDA/PDB | 3-tier dispatch, hybrid vtable routing | `jmp [r*4+t]` | host `fs:` (Win x86 only) | **double, fldcw ignored** | yes | no (PAGEZERO) | MIT | 2026-09-22 | tools/ideas |
| remill (+McSema) | x86/64/AArch64 -> LLVM IR | front end (IDA) | front end | front end | McSema: yes | **double on arm64, PC ignored** | McSema (archived) | lib only | Apache-2 / AGPL | 2026-08-27 / 2022 | semantics lib at most |
| rev.ng | many incl. i386 -> LLVM IR -> C (decompiler) | iterative + PDB import | value analysis | yes | unverified | QEMU softfloat helpers | yes (PECOFF) | no (Linux host) | GPL-2 | 2026-09-22 | second reader |
| RetDec | decompiler | own | own | own | n/a | n/a | yes | runs on mac | MIT | 2026-05-26 (limited maint.) | no |
| BinRec / Polynima | x86 (Linux) traces -> LLVM | dynamic / hybrid | miss handlers | dynamic | no | S2E int-emulated | no | no | ? | 2022 / no source | no |
| Instrew | x86-64/A64/RV64 DBT | dynamic | dynamic | dynamic | n/a | n/a | no | no | LGPL-2.1 | 2026-09-10 | no |
| reccmp | verifier (asm match) | PDB + annotations | vtable check | n/a | n/a | exact by construction (same MSVC) | yes | runs anywhere; output x86 | AGPL-3 | 2026-09-22 | as a method |
| x86port | x86-32 JIT | dynamic | dynamic | dynamic | ? | ext80 softfloat (arm64 path still double) | yes | partial | **none** | 2026-09-22 | no |
| Unicorn | x86 guest, QEMU 5.0.1 TCG | n/a | n/a | n/a | n/a | **floatx80, PC+RC honoured; fsin/fcos/fpatan/f2xm1/fyl2x via host double** | via our loader | **yes (already used here)** | GPL-2 | 2026-08-28 (dev) | **oracle now** |
| FEX | x86/x64 -> arm64 DBT | dynamic | dynamic | dynamic | yes (Linux) | **SoftFloat-3e extF80, PC+RC honoured; cephes transcendentals** | via Wine | **no (Linux)** | MIT | 2026-09-22 | x87 code to borrow |
| Box86/64 | x86/x64 -> arm64 DBT | dynamic | dynamic | dynamic | yes | host double/float | via Wine | no | MIT | 2026-09-22 | no |
| Rosetta 2 | x86/x64 -> arm64 | AOT + JIT | — | — | — | "full, slow, software 80-bit"; PC unverified | via Wine | yes, whole process only | proprietary | — | it *is* the current oracle's FPU |
| Blink | x86-64 (and 16/32) emu | dynamic | — | — | — | not 80-bit | no | yes | ISC | 2025-12-10 | no |

## Answer

**Does a PE32 x86 static recompiler exist that we should build on rather than write?** No, not for this goal. Three things exist and are close:
- **SR/SRW** is mature, MIT, and ships Win32-PE games as native macOS arm64 builds. But it is a whole-program, per-game, assembler-level recompiler with x87 as `double`.
- **pcrecomp** (whose sibling `rol` targets Rise of Legends) has the right hybrid "lifted beside real" plumbing and a Unicorn difftest. But its x87 is `double` with `fldcw` ignored, its SEH relies on the host TIB, it builds for a 32-bit Windows target, and the project is seven months old.
- **remill** has maintained x86 semantics into LLVM IR. But on arm64 its x87 is `double` and it forces precision control to a constant.

None of the three is float-exact, and none is built for "one original function in-process beside Rust". What we have that none of them had is a full private PDB. That removes their hardest problem (discovery, bounds, vtables, switch tables), so the part we would write is small: per-function x86 -> C or LLVM IR emission against a CPU struct, with x87 through an ext80 softfloat. Borrow the *ideas*:
- N64Recomp/XenonRecomp: symbol-driven per-function output, weak-alias replacement, and a lookup table for indirect calls.
- pcrecomp: scoring discovery against the PDB, a Unicorn difftest per function, and hybrid vtable routing.
- SRW: relocation-driven data layout.

Take the x87 arithmetic *code* from FEX's MIT `SoftFloat.h`/`F80Fallbacks.h` or directly from Berkeley SoftFloat-3e. Also treat reccmp-style matching decomp with the original MSVC 7.1 as the zero-risk alternative, which is bit-exact by construction but x86-only.

**Which dynamic translator is the cheapest in-process oracle with bit-exact x87?** **Unicorn.** It is already in this repo's toolchain and running on this Mac. It has a GPL-2 Rust binding. Its QEMU 5.0.1 softfloat honours the x87 precision-control and rounding-control words, with the extended exponent range kept, for arithmetic, sqrt, and loads and stores. Two caveats need handling:
1. Its transcendentals (`fsin`, `fcos`, `fptan`, `fpatan`, `f2xm1`, `fyl2x`) go through host `double` libm. They must be checked in the target functions: grep the decompile for them. If they occur, patch those helpers, for example with upstream QEMU's floatx80 versions or FEX's.
2. The ground truth is Rosetta's x87, not Intel's, because the oracle runs under Wine+Rosetta. So the first spike measurement should be a Unicorn-vs-dump agreement test on one float-heavy function.

A cheap first check: Windows processes run the x87 at PC=53 by default (x87sidecar README: "the 53-bit precision Windows processes run at"). If RoN never raises PC, then a double-based lifter is exact for + - * / except where the 15-bit exponent range or `fld/fstp m80` matter. A Direct3D 9 device created without `D3DCREATE_FPU_PRESERVE` drops PC to 24 (**unverified for RoN**). The PC value should be read from the executable's `fldcw` sites before the lifter's float model is chosen. FEX has the best x87 of the translators, but it is Linux-only. Box64 and Blink are not 80-bit. Rosetta is exact-by-claim, but it translates whole processes only.
