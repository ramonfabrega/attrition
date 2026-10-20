//! `Army::find_target@006f69b0` under the emulator against the port (item 3362; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `Army::find_target@006f69b0`, run by `tools/emu/sweep_army_find_target_run.py` under unicorn
/// (tools/emu): 152 random bytes of object, 0 argument(s), `p` windows in order: p0..p119 sparse table 0 (8 rows × 15 slice(s), row-major), p120..p135 sparse table 1 (8 rows × 2 slice(s), row-major), p136..p288 the place(s), in flag order; the global cell 0x00c061e8 points at 2112 random bytes; the global cell 0x00c061ec points at 1536 random bytes; the global cell 0x00c061b8 points at 32 random bytes; the global cell 0x00c06188 points at 312 random bytes; the global cell 0x00c0618c points at 528 random bytes; callee 0x006f9120 (2 args) stubbed; callee 0x006e33a0 (2 args) stubbed; callee 0x006eb930 (4 args) stubbed; callee 0x006d5b90 (1 args) stubbed; callee 0x006f2e10 (1 args) stubbed; callee 0x006ec000 (0 args) stubbed; callee 0x00680f90 (1 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x0062d470 (0 args) stubbed; callee 0x00739340 (0 args) stubbed; callee 0x007382b0 (1 args) stubbed; callee 0x005948a0 (0 args) stubbed; callee 0x006db810 (1 args) stubbed; callee 0x006f2c90 (5 args) stubbed; callee 0x006f5cc0 (3 args) stubbed; callee 0x006f8ea0 (0 args) stubbed; callee 0x006f9df0 (1 args) stubbed; vtable slot +0x3c (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x11c (1 args) stubbed; the word at `0xc09970+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p136+0` points at 112 random bytes less `0` (placed); the word at `p136+4` points at 112 random bytes less `0` (placed); the word at `p136+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p140+0` points at 112 random bytes less `0` (placed); the word at `p140+4` points at 112 random bytes less `0` (placed); the word at `p140+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*2` points at 12 random bytes less `0` (placed); the word at `p144+0` points at 112 random bytes less `0` (placed); the word at `p144+4` points at 112 random bytes less `0` (placed); the word at `p144+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*3` points at 12 random bytes less `0` (placed); the word at `p148+0` points at 112 random bytes less `0` (placed); the word at `p148+4` points at 112 random bytes less `0` (placed); the word at `p148+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*4` points at 12 random bytes less `0` (placed); the word at `p152+0` points at 112 random bytes less `0` (placed); the word at `p152+4` points at 112 random bytes less `0` (placed); the word at `p152+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*5` points at 12 random bytes less `0` (placed); the word at `p156+0` points at 112 random bytes less `0` (placed); the word at `p156+4` points at 112 random bytes less `0` (placed); the word at `p156+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*6` points at 12 random bytes less `0` (placed); the word at `p160+0` points at 112 random bytes less `0` (placed); the word at `p160+4` points at 112 random bytes less `0` (placed); the word at `p160+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*7` points at 12 random bytes less `0` (placed); the word at `p164+0` points at 112 random bytes less `0` (placed); the word at `p164+4` points at 112 random bytes less `0` (placed); the word at `p164+8` points at 112 random bytes less `0` (placed); the word at `c4+0x14+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p168+0` points at 64 random bytes less `0` (placed); the word at `p168+4` points at 64 random bytes less `0` (placed); the word at `p168+8` points at 64 random bytes less `0` (placed); the word at `c4+0x14+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p172+0` points at 64 random bytes less `0` (placed); the word at `p172+4` points at 64 random bytes less `0` (placed); the word at `p172+8` points at 64 random bytes less `0` (placed); the word at `c4+0x14+0x1c*2` points at 12 random bytes less `0` (placed); the word at `p176+0` points at 64 random bytes less `0` (placed); the word at `p176+4` points at 64 random bytes less `0` (placed); the word at `p176+8` points at 64 random bytes less `0` (placed); the word at `c4+0x14+0x1c*3` points at 12 random bytes less `0` (placed); the word at `p180+0` points at 64 random bytes less `0` (placed); the word at `p180+4` points at 64 random bytes less `0` (placed); the word at `p180+8` points at 64 random bytes less `0` (placed); the word at `c4+0x14+0x1c*4` points at 12 random bytes less `0` (placed); the word at `p184+0` points at 64 random bytes less `0` (placed); the word at `p184+4` points at 64 random bytes less `0` (placed); the word at `p184+8` points at 64 random bytes less `0` (placed); the word at `c4+0x14+0x1c*5` points at 12 random bytes less `0` (placed); the word at `p188+0` points at 64 random bytes less `0` (placed); the word at `p188+4` points at 64 random bytes less `0` (placed); the word at `p188+8` points at 64 random bytes less `0` (placed); the word at `c4+0x14+0x1c*6` points at 12 random bytes less `0` (placed); the word at `p192+0` points at 64 random bytes less `0` (placed); the word at `p192+4` points at 64 random bytes less `0` (placed); the word at `p192+8` points at 64 random bytes less `0` (placed); the word at `c4+0x14+0x1c*7` points at 12 random bytes less `0` (placed); the word at `p196+0` points at 64 random bytes less `0` (placed); the word at `p196+4` points at 64 random bytes less `0` (placed); the word at `p196+8` points at 64 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p200+0` points at 64 random bytes less `0` (placed); the word at `p200+4` points at 64 random bytes less `0` (placed); the word at `p200+8` points at 64 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p204+0` points at 64 random bytes less `0` (placed); the word at `p204+4` points at 64 random bytes less `0` (placed); the word at `p204+8` points at 64 random bytes less `0` (placed); the word at `0xc09710+0x1c*0` points at 64 random bytes less `0` (placed); the word at `p208+0` points at 80 random bytes less `0` (placed); the word at `p208+4` points at 80 random bytes less `0` (placed); the word at `p208+8` points at 80 random bytes less `0` (placed); the word at `p208+12` points at 80 random bytes less `0` (placed); the word at `p208+16` points at 80 random bytes less `0` (placed); the word at `p208+20` points at 80 random bytes less `0` (placed); the word at `p208+24` points at 80 random bytes less `0` (placed); the word at `p208+28` points at 80 random bytes less `0` (placed); the word at `p208+32` points at 80 random bytes less `0` (placed); the word at `p208+36` points at 80 random bytes less `0` (placed); the word at `p208+40` points at 80 random bytes less `0` (placed); the word at `p208+44` points at 80 random bytes less `0` (placed); the word at `p208+48` points at 80 random bytes less `0` (placed); the word at `p208+52` points at 80 random bytes less `0` (placed); the word at `p208+56` points at 80 random bytes less `0` (placed); the word at `p208+60` points at 80 random bytes less `0` (placed); the word at `0xc09710+0x1c*1` points at 64 random bytes less `0` (placed); the word at `p225+0` points at 80 random bytes less `0` (placed); the word at `p225+4` points at 80 random bytes less `0` (placed); the word at `p225+8` points at 80 random bytes less `0` (placed); the word at `p225+12` points at 80 random bytes less `0` (placed); the word at `p225+16` points at 80 random bytes less `0` (placed); the word at `p225+20` points at 80 random bytes less `0` (placed); the word at `p225+24` points at 80 random bytes less `0` (placed); the word at `p225+28` points at 80 random bytes less `0` (placed); the word at `p225+32` points at 80 random bytes less `0` (placed); the word at `p225+36` points at 80 random bytes less `0` (placed); the word at `p225+40` points at 80 random bytes less `0` (placed); the word at `p225+44` points at 80 random bytes less `0` (placed); the word at `p225+48` points at 80 random bytes less `0` (placed); the word at `p225+52` points at 80 random bytes less `0` (placed); the word at `p225+56` points at 80 random bytes less `0` (placed); the word at `p225+60` points at 80 random bytes less `0` (placed); the word at `0xc09710+0x1c*2` points at 64 random bytes less `0` (placed); the word at `p242+0` points at 80 random bytes less `0` (placed); the word at `p242+4` points at 80 random bytes less `0` (placed); the word at `p242+8` points at 80 random bytes less `0` (placed); the word at `p242+12` points at 80 random bytes less `0` (placed); the word at `p242+16` points at 80 random bytes less `0` (placed); the word at `p242+20` points at 80 random bytes less `0` (placed); the word at `p242+24` points at 80 random bytes less `0` (placed); the word at `p242+28` points at 80 random bytes less `0` (placed); the word at `p242+32` points at 80 random bytes less `0` (placed); the word at `p242+36` points at 80 random bytes less `0` (placed); the word at `p242+40` points at 80 random bytes less `0` (placed); the word at `p242+44` points at 80 random bytes less `0` (placed); the word at `p242+48` points at 80 random bytes less `0` (placed); the word at `p242+52` points at 80 random bytes less `0` (placed); the word at `p242+56` points at 80 random bytes less `0` (placed); the word at `p242+60` points at 80 random bytes less `0` (placed); the word at `0xc09710+0x1c*3` points at 64 random bytes less `0` (placed); the word at `p259+0` points at 80 random bytes less `0` (placed); the word at `p259+4` points at 80 random bytes less `0` (placed); the word at `p259+8` points at 80 random bytes less `0` (placed); the word at `p259+12` points at 80 random bytes less `0` (placed); the word at `p259+16` points at 80 random bytes less `0` (placed); the word at `p259+20` points at 80 random bytes less `0` (placed); the word at `p259+24` points at 80 random bytes less `0` (placed); the word at `p259+28` points at 80 random bytes less `0` (placed); the word at `p259+32` points at 80 random bytes less `0` (placed); the word at `p259+36` points at 80 random bytes less `0` (placed); the word at `p259+40` points at 80 random bytes less `0` (placed); the word at `p259+44` points at 80 random bytes less `0` (placed); the word at `p259+48` points at 80 random bytes less `0` (placed); the word at `p259+52` points at 80 random bytes less `0` (placed); the word at `p259+56` points at 80 random bytes less `0` (placed); the word at `p259+60` points at 80 random bytes less `0` (placed); the word at `0xcae5fc` points at 1024 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*0+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*1+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*2+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*3+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*4+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*5+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*6+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*7+0x6eb8` points at 256 random bytes less `0` (placed); the word at `c3+0x134` points at 12288 random bytes less `0` (placed); stub 17's answer points at 256 random bytes less `0` (placed); the word at `p286+0x18` points at 32 random bytes less `0` (placed); stub 18's answer points at 256 random bytes less `0` (placed); stub 2 answered from [0] (90%); the stubbed vtable laid at `m+0x90`; stub 0 answers 6 values in call order, printed after the answers; stub 3 answers 16 values in call order, printed after the answers; stub 4 answers 16 values in call order, printed after the answers; stub 5 answers 16 values in call order, printed after the answers; stub 6 answers 48 values in call order, printed after the answers; stub 7 answers 40 values in call order, printed after the answers; stub 8 answers 8 values in call order, printed after the answers; stub 9 answers 6 values in call order, printed after the answers; stub 10 answers 4 values in call order, printed after the answers; stub 19 answers 2 values in call order, printed after the answers; stub 0 answers 0..6 every row; stub 1 answers 0..1 every row; stub 3 answers 0..1 every row; stub 4 answers -1..7 every row; stub 5 answers 0..5 every row; stub 6 answers 0..1 every row; stub 7 answers 0..65535 every row; stub 8 answers 0..1 every row; stub 9 answers 1..3 every row; stub 10 answers 0..1 every row; stub 11 answers 0..7 every row; stub 12 answers 0..1 every row; stub 14 answers 0..1 every row; stub 16 answers -1..-1 every row; stub 19 answers 0..100 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 2 writes a value in [-1, 0, 1, 2] through its argument 0, printed after the answers (`o` in a place); stub 2 writes a value in [0, 1, 2, 3, 4, 5, 6, 7] through its argument 1, printed after the answers (`o` in a place); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x8:0x4, +0x18:0x4, +0x50:0x4, +0x74:0x20, +0x230:0x20, +0x250:0x20, +0x408:0x4, +0x444:0xc, +0x7a8:0x4, +0x7e8:0x4, +0x944:0xc, +0xa40:0x8, +0x6dec:0x4, +0x6e20:0x4 (`p` windows after the pointed buffers, before the places); 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4, +0x74:0x20 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p288+0` as well as at `this`; the stubbed vtable laid at `p169+0` as well as at `this`; the stubbed vtable laid at `p170+0` as well as at `this`; the stubbed vtable laid at `p171+0` as well as at `this`; the stubbed vtable laid at `p173+0` as well as at `this`; the stubbed vtable laid at `p174+0` as well as at `this`; the stubbed vtable laid at `p175+0` as well as at `this`; the stubbed vtable laid at `p177+0` as well as at `this`; the stubbed vtable laid at `p178+0` as well as at `this`; the stubbed vtable laid at `p179+0` as well as at `this`; the stubbed vtable laid at `p181+0` as well as at `this`; the stubbed vtable laid at `p182+0` as well as at `this`; the stubbed vtable laid at `p183+0` as well as at `this`; the stubbed vtable laid at `p185+0` as well as at `this`; the stubbed vtable laid at `p186+0` as well as at `this`; the stubbed vtable laid at `p187+0` as well as at `this`; the stubbed vtable laid at `p189+0` as well as at `this`; the stubbed vtable laid at `p190+0` as well as at `this`; the stubbed vtable laid at `p191+0` as well as at `this`; the stubbed vtable laid at `p193+0` as well as at `this`; the stubbed vtable laid at `p194+0` as well as at `this`; the stubbed vtable laid at `p195+0` as well as at `this`; the stubbed vtable laid at `p197+0` as well as at `this`; the stubbed vtable laid at `p198+0` as well as at `this`; the stubbed vtable laid at `p199+0` as well as at `this`; the stubbed vtable laid at `p201+0` as well as at `this`; the stubbed vtable laid at `p202+0` as well as at `this`; the stubbed vtable laid at `p203+0` as well as at `this`; the stubbed vtable laid at `p205+0` as well as at `this`; the stubbed vtable laid at `p206+0` as well as at `this`; the stubbed vtable laid at `p207+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_find_target_run.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            use crate::target_gate::{
                Ecx, Stopped, TargetGateView, target_city_gate, target_head, target_leader_gate,
            };
            use crate::target_score::{
                Ctx, TargetScoreView, TargetTailView, target_score, target_tail,
            };
            // Windows: `m` the army (`+2` slot, `+8` region, `+0x14` captains, `+0x18` standard,
            // `+0x24` navy, `+0x2c` hurry, `+0x30`/`+0x34` target, `+0x38`/`+0x3c` point, `+0x44`
            // rally, `+0x50` angle, `+0x94` who); `c[0]` gamec, `c[1]` game, `c[2]` the regions cell,
            // `c[3]` the world (`+0` width, `+4` height), `c[4]` the objects cell;
            // `p[15r+s]` slice `s` of `leaders[r]`, in offset order: 0 flags, 1 id, 2 `+0x18`,
            // 3 multi_diff, 4 diplos, 5 `+0x230`, 6 `+0x250`, 7 cities, 8 `+0x444..`, 9 `+0x7a8`,
            // 10 `+0x7e8`, 11 `+0x944..`, 12 `+0xa40..`, 13 `+0x6dec`, 14 `+0x6e20`;
            // `p[120+2r]`/`p[121+2r]` the `leadersc` row's flags and diplos; `p[136+4a]` the city
            // array of leader `a`, `p[137+4a+j]` its city `j`; `p[168+4a]` its building objects,
            // `p[169+4a+j]`; `p[200+4t]` the type objects, `p[201+4t+j]`; `p[208+17a]` the armies of
            // leader `a`, `p[209+17a+j]`; `p[276]` the `div_3_table`; `p[277+a]` the record
            // `leaders[a]+0x6eb8` names; `p[285]` the world's cell table; `p[286]` the `BuildData`
            // slot 0x3c answers, `p[287]` the record its `+0x18` names (`+4` the kind), `p[288]`
            // what slot 0xac answers (`+0x24` the damage).
            // `a`: the twenty stubs' answers, `find_capital`'s two outs, then the call-order answers
            // of stubs 0 (6), 3 (16), 4 (16), 5 (16), 6 (48), 7 (40), 8 (8), 9 (6), 10 (4), 19 (2).
            const OUT: usize = 20;
            const SEQ: [(usize, usize); 10] = [
                (0, 22),
                (3, 28),
                (4, 44),
                (5, 60),
                (6, 76),
                (7, 124),
                (8, 164),
                (9, 172),
                (10, 178),
                (19, 182),
            ];
            const LEN: [usize; 10] = [6, 16, 16, 16, 48, 40, 8, 6, 4, 2];
            struct W<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                n: [usize; 20],
            }
            impl W<'_> {
                fn lw(&self, a: usize, s: usize, off: usize) -> i32 {
                    rd(&self.p[15 * a + s], off)
                }
                fn leader(&self) -> String {
                    format!("p{}+0x0", 15 * self.who())
                }
                fn rec(&self, a: usize, k: usize) -> &[u8] {
                    &self.p[137 + 4 * a + k]
                }
                fn city_tok(&self, a: usize, k: usize) -> String {
                    format!("p{}+0x0", 137 + 4 * a + k)
                }
                fn id_of(&self, a: usize, k: usize) -> usize {
                    let b = self.rec(a, k);
                    i16::from_le_bytes([b[8], b[9]]) as usize
                }
                fn seq(&mut self, stub: usize) -> i32 {
                    let at = SEQ
                        .iter()
                        .position(|s| s.0 == stub)
                        .expect("a stub with a sequence");
                    let k = self.n[stub].min(LEN[at] - 1);
                    self.n[stub] += 1;
                    self.a[SEQ[at].1 + k]
                }
                fn div3(&self, v: i32) -> i32 {
                    rd(&self.p[276], 4 * (v >> 8) as usize)
                }
                fn region_at(&self, x: i32, y: i32) -> i32 {
                    let idx = self.div3(y) * rd(&self.c[3], 0) + self.div3(x);
                    let t = &self.p[285];
                    let at = 28 * idx as usize + 4;
                    i32::from(i16::from_le_bytes([t[at], t[at + 1]]))
                }
                fn bld(&self, leader: usize, id: i32) -> &[u8] {
                    &self.p[169 + 4 * leader + id as usize]
                }
            }
            impl TargetGateView for W<'_> {
                fn slot(&self) -> i32 {
                    i32::from(i16::from_le_bytes([self.m[2], self.m[3]]))
                }
                fn region(&self) -> i32 {
                    rd(self.m, 8)
                }
                fn captains(&self) -> i32 {
                    rd(self.m, 0x14)
                }
                fn standard(&self) -> i32 {
                    rd(self.m, 0x18)
                }
                fn navy(&self) -> bool {
                    rd(self.m, 0x24) != 0
                }
                fn who(&self) -> usize {
                    let w = i16::from_le_bytes([self.m[0x94], self.m[0x95]]);
                    assert!((0..4).contains(&w), "the sweep keeps who in the table");
                    w as usize
                }
                fn lflags(&self, a: usize) -> i32 {
                    self.lw(a, 0, 0)
                }
                fn lid(&self, a: usize) -> i32 {
                    self.lw(a, 1, 0)
                }
                fn lmulti(&self, a: usize) -> i32 {
                    self.lw(a, 3, 0)
                }
                fn ld(&self, a: usize, b: usize) -> i32 {
                    self.lw(a, 4, 4 * b)
                }
                fn lc(&self, a: usize, b: i32) -> i32 {
                    rd(&self.p[121 + 2 * a], 4 * b as usize)
                }
                fn lcflags(&self, a: i32) -> i32 {
                    rd(&self.p[120 + 2 * a as usize], 0)
                }
                fn x230(&self, a: usize, b: usize) -> i32 {
                    self.lw(a, 5, 4 * b)
                }
                fn x250(&self, a: usize, b: usize) -> i32 {
                    self.lw(a, 6, 4 * b)
                }
                fn ncities(&self, a: usize) -> i32 {
                    self.lw(a, 7, 0)
                }
                fn w444(&self, a: usize) -> i32 {
                    self.lw(a, 8, 0)
                }
                fn w44c(&self, a: usize) -> i32 {
                    self.lw(a, 8, 8)
                }
                fn raid(&self, a: usize) -> i32 {
                    self.lw(a, 13, 0)
                }
                fn early(&self, a: usize) -> i32 {
                    self.lw(a, 14, 0)
                }
                fn defmod(&self, a: usize) -> i32 {
                    self.lw(a, 9, 0)
                }
                fn combat(&self, a: usize) -> i32 {
                    self.lw(a, 11, 0)
                }
                fn sea(&self, a: usize) -> i32 {
                    self.lw(a, 11, 8)
                }
                fn frame_attacked(&self, a: usize) -> i32 {
                    self.lw(a, 12, 0)
                }
                fn attacked_by(&self, a: usize) -> i32 {
                    self.lw(a, 12, 4)
                }
                fn military(&self, a: usize) -> i32 {
                    rd(&self.p[277 + a], 0xe8) ^ 0x63187
                }
                fn semaphore(&self) -> [u8; 3] {
                    [self.c[0][0x820], self.c[0][0x821], self.c[0][0x822]]
                }
                fn game_difficulty(&self) -> i32 {
                    i32::from(self.c[0][0x2b])
                }
                fn team_style(&self) -> i32 {
                    i32::from(self.c[1][0x24])
                }
                fn frame(&self) -> i32 {
                    rd(&self.c[1], 0x550)
                }
                fn start_list(&self, k: i32) -> i32 {
                    rd(&self.c[0], 0x65c + 4 * k as usize)
                }
                fn start_index(&self, id: i32) -> i32 {
                    rd(&self.c[0], 0x67c + 4 * id as usize)
                }
                fn city_flags(&self, a: usize, k: usize) -> i32 {
                    rd(self.rec(a, k), 4)
                }
                fn city_reg(&self, a: usize, k: usize) -> i32 {
                    let b = self.rec(a, k);
                    i32::from(i16::from_le_bytes([b[0xa], b[0xb]]))
                }
                fn city_founder(&self, a: usize, k: usize) -> i32 {
                    i32::from(self.rec(a, k)[0x60])
                }
                fn city_ocean(&self, a: usize, k: usize) -> i32 {
                    i32::from(self.rec(a, k)[0x62])
                }
                fn capital_cell(&self, owner: i32, idx: i32) -> (i32, i32) {
                    let b = self.rec(owner as usize, idx as usize);
                    (self.div3(rd(b, 0xc)), self.div3(rd(b, 0x10)))
                }
                fn count(&mut self, kind: i32, arg: i32) -> i32 {
                    self.log.push(format!("s0:m+0x0,{kind},{arg}"));
                    self.seq(0)
                }
                fn type_avail(&mut self, ty: i32, strict: i32) -> i32 {
                    self.log.push(format!("s1:{},{ty},{strict}", self.leader()));
                    self.a[1]
                }
                fn find_capital(&mut self) -> (i32, i32) {
                    self.log
                        .push(format!("s2:{},sp-0x38,sp-0x24,-1,-1", self.leader()));
                    (self.a[OUT], self.a[OUT + 1])
                }
                fn tribute(&mut self, b: usize) -> bool {
                    self.log.push(format!("s3:{},{b}", self.leader()));
                    self.seq(3) != 0
                }
                fn find_aggressive(&mut self, ecx: Ecx) -> i32 {
                    let t = match ecx {
                        Ecx::Army => "m+0x0".to_string(),
                        Ecx::Num(n) => n.to_string(),
                    };
                    self.log.push(format!("s4:{t},{}", self.who()));
                    self.seq(4)
                }
                fn get_diff(&mut self) -> i32 {
                    self.log.push(format!("s5:{}", self.leader()));
                    self.seq(5)
                }
                fn is_coast(&mut self, reg: i32, city_reg: i32) -> bool {
                    let ecx = rd(&self.c[2], 0x10).wrapping_add(reg.wrapping_mul(0x88));
                    self.log.push(format!("s6:{ecx},{city_reg}"));
                    self.seq(6) != 0
                }
                fn draw(&mut self) -> Result<i32, Stopped> {
                    // The image's word at 0xc06184 (`Random`'s object), then `(0, 0xffff)`.
                    self.log.push("s7:14908044,0,65535".to_string());
                    Ok(self.seq(7))
                }
            }
            impl TargetScoreView for W<'_> {
                fn f7e8(&self, a: usize) -> i32 {
                    self.lw(a, 10, 0)
                }
                fn f18(&self, a: usize) -> i32 {
                    self.lw(a, 2, 0)
                }
                fn transport(&self, a: usize) -> i32 {
                    (self.lw(a, 0, 0) >> 8) & 7
                }
                fn world_width(&self) -> i32 {
                    rd(&self.c[3], 0)
                }
                fn city_id(&self, a: usize, k: usize) -> i32 {
                    self.id_of(a, k) as i32
                }
                fn city_bits(&self, a: usize, k: usize) -> i32 {
                    i32::from(u16::from_le_bytes([self.rec(a, k)[4], self.rec(a, k)[5]]))
                }
                fn city_bordering(&self, a: usize, k: usize) -> i32 {
                    i32::from(self.rec(a, k)[0x65])
                }
                fn city_was_capital(&self, a: usize, k: usize) -> i32 {
                    i32::from(self.rec(a, k)[0x68])
                }
                fn city_stamp(&self, a: usize, k: usize) -> i32 {
                    rd(self.rec(a, k), 0x14)
                }
                fn city_cell(&self, a: usize, k: usize) -> (i32, i32) {
                    let b = self.rec(a, k);
                    (self.div3(rd(b, 0xc)), self.div3(rd(b, 0x10)))
                }
                fn army_open(&self, j: usize) -> Option<(i32, i32)> {
                    let b = &self.p[209 + 17 * self.who() + j];
                    if i16::from_le_bytes([b[0], b[1]]) == 0 || b[4] & 1 != 0 {
                        return None;
                    }
                    Some((rd(b, 0x30), rd(b, 0x34)))
                }
                fn city_kind(&mut self, a: usize, k: usize) -> i32 {
                    let ty = self.rec(a, k)[0x5e] as i8 as usize;
                    let id = self.id_of(a, k);
                    self.log.push(format!("s17:p{}+0x0", 201 + 4 * ty + id));
                    rd(&self.p[287], 4)
                }
                fn unassimilated(&mut self, a: usize, k: usize) -> bool {
                    let _ = self.city_kind(a, k);
                    self.log.push("s8:p286+0x0".to_string());
                    self.seq(8) != 0
                }
                fn city_level(&mut self, a: usize, k: usize) -> i32 {
                    let t = self.city_tok(a, k);
                    self.log.push(format!("s9:{t}"));
                    self.seq(9)
                }
                fn num_wonders(&mut self, a: usize, k: usize) -> i32 {
                    let t = self.city_tok(a, k);
                    self.log.push(format!("s10:{t},0"));
                    self.seq(10)
                }
                fn wonder_winning(&mut self) -> i32 {
                    // `ecx` is still the city `num_wonders` was handed.
                    let t = self.log.last().expect("num_wonders ran").clone();
                    let t = t
                        .trim_start_matches("s10:")
                        .split(',')
                        .next()
                        .expect("token")
                        .to_string();
                    self.log.push(format!("s11:{t}"));
                    self.a[11]
                }
                fn has_preq(&mut self) -> bool {
                    self.log.push(format!("s12:{},697", self.leader()));
                    self.a[12] != 0
                }
                fn building_damage(&mut self, a: usize, k: usize) -> i32 {
                    let id = self.id_of(a, k) as i32;
                    self.log
                        .push(format!("s18:p{}+0x0", 169 + 4 * a + id as usize));
                    rd(&self.p[288], 0x24)
                }
                fn building_hits(&mut self, a: usize, k: usize) -> i32 {
                    let id = self.id_of(a, k) as i32;
                    self.log
                        .push(format!("s18:p{}+0x0", 169 + 4 * a + id as usize));
                    self.log.push("s19:p288+0x0,0".to_string());
                    self.seq(19)
                }
            }
            impl TargetTailView for W<'_> {
                fn target_is(&self, city: i32, leader: usize) -> bool {
                    rd(self.m, 0x30) == city && rd(self.m, 0x34) == leader as i32
                }
                fn army_cell_region(&self) -> i32 {
                    self.region_at(rd(self.m, 0x38), rd(self.m, 0x3c))
                }
                fn target_cell_region(&self, leader: usize, city: i32) -> i32 {
                    let b = self.bld(leader, city);
                    self.region_at(rd(b, 0x10) ^ 0x63637, rd(b, 0x14) ^ 0x63637)
                }
                fn set_hurry0(&mut self) {
                    wr(self.m, 0x2c, 0);
                }
                fn set_rally(&mut self, v: i32) {
                    wr(self.m, 0x44, v);
                }
                fn set_target(&mut self, city: i32, leader: usize) {
                    wr(self.m, 0x30, city);
                    wr(self.m, 0x34, leader as i32);
                }
                fn mark_attacked(&mut self, leader: usize) {
                    let frame = rd(&self.c[1], 0x550);
                    let who = self.who() as i32;
                    wr(&mut self.p[15 * leader + 12], 0, frame);
                    wr(&mut self.p[15 * leader + 12], 4, who);
                }
                fn send_navy(&mut self, _city: i32, _leader: usize, ra: i32, rb: i32) {
                    let t = (rd(self.m, 0x30), rd(self.m, 0x34));
                    self.log
                        .push(format!("s13:{rb},{},{},{},{ra},{rb}", self.who(), t.0, t.1));
                }
                fn set_point(&mut self, leader: usize, city: i32) {
                    let b = self.bld(leader, city);
                    let (mut x, mut y) = (rd(b, 0x10) ^ 0x63637, (rd(b, 0x14) ^ 0x63637) + 0x300);
                    if x < 0 {
                        x = 0;
                    }
                    if y < 0 {
                        y = 0;
                    }
                    let (w, h) = (rd(&self.c[3], 0), rd(&self.c[3], 4));
                    if x >= w * 0x300 {
                        x = w * 0x300 - 1;
                    }
                    if y >= h * 0x300 {
                        y = h * 0x300 - 1;
                    }
                    wr(self.m, 0x38, x);
                    wr(self.m, 0x3c, y);
                }
                fn muster_spot(&mut self, city: i32, leader: usize) -> bool {
                    self.log.push(format!("s14:m+0x0,{city},{leader},0"));
                    self.a[14] != 0
                }
                fn close_army(&mut self) {
                    self.log.push("s15:m+0x0".to_string());
                }
                fn flip_angle(&mut self) {
                    let v = rd(self.m, 0x50).wrapping_add(i32::MIN);
                    wr(self.m, 0x50, v);
                }
                fn probe(&mut self, _city: i32, _leader: usize) -> bool {
                    self.log.push("s16:m+0x0,0".to_string());
                    self.a[16] < 0
                }
            }
            let mut w = W {
                a,
                m,
                c,
                p,
                log: &mut log,
                n: [0; 20],
            };
            // The whole of `find_target` but the forts pass (the sweep keeps the army's groups at
            // none) and the probe's body (the sweep answers its first call `-1`).
            let head = target_head(&mut w);
            if !head.stop {
                let mut best: Option<(i32, usize, i32)> = None;
                'run: for pass in 0..2 {
                    for i in 0..8 {
                        let about = match target_leader_gate(&mut w, i) {
                            Err(Stopped) => break 'run,
                            Ok(None) => continue,
                            Ok(Some(about)) => about,
                        };
                        for k in 0..w.ncities(i) as usize {
                            if !target_city_gate(&mut w, i, k) {
                                continue;
                            }
                            let draw = w.draw().expect("the sweep's draws do not stop");
                            let ctx = Ctx {
                                strong: head.strong,
                                cap: head.cap,
                                ours: head.ours,
                                theirs: head.theirs,
                                supply: head.supply,
                                pass,
                                about,
                            };
                            let Some(v) = target_score(&mut w, &ctx, i, k, draw) else {
                                continue;
                            };
                            if best.is_none_or(|(bv, _, _)| bv <= v) {
                                best = Some((v, i, w.city_id(i, k)));
                            }
                        }
                    }
                    if best.is_some() {
                        break;
                    }
                }
                if let Some((_, leader, id)) = best {
                    target_tail(&mut w, leader, id);
                }
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "find_target — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_target — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_target — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "find_target — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Army::find_target@006f69b0: {n} rows agree");
}
