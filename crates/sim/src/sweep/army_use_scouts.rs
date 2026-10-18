//! `Army::use_scouts@006f49a0` under the emulator against the port (item 3219; tools/sweep_gen.py, obj with windows).

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

/// `Army::use_scouts@006f49a0`, run by `tools/emu/sweep_army_use_scouts.py` under unicorn
/// (tools/emu): 160 random bytes of object, 0 argument(s), vtable slot +0x8 (0 args) stubbed; vtable slot +0xd4 (0 args) stubbed; callee 0x005f27a0 (0 args) stubbed; the word at `0xc0618c` points at 256 random bytes less `0` (placed); the word at `p6+0x14+0x1c*m[0x94]` points at 16 random bytes less `0` (placed); the word at `p7+0` points at 16 random bytes less `0` (placed); the word at `p7+4` points at 16 random bytes less `0` (placed); the word at `p7+8` points at 16 random bytes less `0` (placed); the word at `p7+12` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*m[0x94]` points at 16 random bytes less `0` (placed); the word at `p12+0` points at 32 random bytes less `0` (placed); the word at `p12+4` points at 32 random bytes less `0` (placed); the word at `p12+8` points at 32 random bytes less `0` (placed); the word at `p12+12` points at 32 random bytes less `0` (placed); the word at `p13+0x18` points at 4 random bytes less `0x2b8` (placed); the word at `p14+0x18` points at 4 random bytes less `0x2b8` (placed); the word at `p15+0x18` points at 4 random bytes less `0x2b8` (placed); the word at `p16+0x18` points at 4 random bytes less `0x2b8` (placed); slot +0xd4 laid as 0x0046cea0 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 0, 1] (90%); stub 0 answers 12 values in call order, printed after the answers; stub 1 answers 12 values in call order, printed after the answers; stub 2 answers 12 values in call order, printed after the answers; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 3 rows 0x9d4 apart at `0xe85f20`, slices +0xc:0x4, +0x8cc:0x8 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_use_scouts.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The army's group ids `+0x54` (`+0x96` of them), each group's
            // count `p[2g]` and member shorts `p[2g + 1]` (the groups table
            // through `0xe85f20`); in use (vslot `+8`) answers `a[3..15]`,
            // `is_hero` (`+0xd4`) `a[15..27]` or is inline (`a[39]`, the
            // base laid: `type +0x2b8 & 0x20`, `p[17 + o]`), and
            // `think_spellcaster` `a[27..39]`, each in call order. The log:
            // each cast's `ecx`, the unit record `p[13 + o]`.
            let mut members = Vec::new();
            for i in 0..i16::from_le_bytes([m[0x96], m[0x97]]).max(0) as usize {
                let g = rd(m, 0x54 + 4 * i);
                if g < 0 {
                    continue;
                }
                let g = g as usize;
                for k in 0..rd(&p[2 * g], 0).max(0) as usize {
                    let o = i16::from_le_bytes([p[2 * g + 1][2 * k], p[2 * g + 1][2 * k + 1]]);
                    if o >= 0 {
                        members.push(o as usize);
                    }
                }
            }
            let (mut used, mut hero, mut cast) = (0usize, 0usize, Vec::new());
            let pp = p.to_vec();
            crate::army::generals_walk(
                &mut (),
                members,
                |_, _| {
                    used += 1;
                    a[3 + (used - 1).min(11)] != 0
                },
                |_, o| {
                    if a[39] != 0 {
                        rd(&pp[17 + o], 0) & 0x10 != 0
                    } else {
                        hero += 1;
                        a[15 + (hero - 1).min(11)] != 0
                    }
                },
                |_, o| {
                    cast.push(format!("p{}+0x0", 13 + o));
                    a[27 + (cast.len() - 1).min(11)] != 0
                },
            );
            log.push(if cast.is_empty() {
                "-".into()
            } else {
                cast.join("/")
            });
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "use_scouts — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "use_scouts — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "use_scouts — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "use_scouts — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Army::use_scouts@006f49a0: {n} rows agree");
}
