//! `Build::check_gatherers@0062f710` under the emulator against the port (item 2287; tools/sweep_gen.py, obj with windows).

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

/// `Build::check_gatherers@0062f710`, run by `tools/emu/sweep_build_check_gatherers.py` under unicorn
/// (tools/emu): 128 random bytes of object, 0 argument(s), vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x00608880 (3 args) stubbed; the word at `0xc0618c` points at 256 random bytes less `0` (placed); the word at `p0+0x14+0x1c*m[9]` points at 24 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); the word at `p1+16` points at 16 random bytes less `0` (placed); the word at `p1+20` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*m[9]` points at 24 random bytes less `0` (placed); the word at `p8+0` points at 152 random bytes less `0` (placed); the word at `p8+4` points at 152 random bytes less `0` (placed); the word at `p8+8` points at 152 random bytes less `0` (placed); the word at `p8+12` points at 152 random bytes less `0` (placed); the word at `p8+16` points at 152 random bytes less `0` (placed); the word at `p8+20` points at 152 random bytes less `0` (placed); stub 0 answered from [0, 1, 1, 1] (90%); stub 1 answered from [0, 1, 1, 1] (90%); stub 2 answered from [0, 1, 1] (90%); stub 0 answers 12 values in call order, printed after the answers; stub 1 answers 6 values in call order, printed after the answers; stub 2 answers 6 values in call order, printed after the answers; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; figures p9..p14 link at +0x92 (2 bytes) above their own index or to -1, -2; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_check_gatherers.py") else {
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
            // The chain's head `+0x70`, each member k's unit record `p[9 + k]`
            // and its link `+0x92`; vslot `+8` (the object, then the unit
            // record) answers `a[3..15]`, `+0xbc` `a[15..21]`,
            // `is_gathering_at` `a[21..27]`, each in call order. The log:
            // each `is_gathering_at`'s `ecx,o,who,0` — the building's own
            // `+0xa` and `+9`.
            let o = i16::from_le_bytes([m[0xa], m[0xb]]);
            let who = m[9];
            let (mut eight, mut bc, mut gath) = (0usize, 0usize, Vec::new());
            let mut head = i32::from(i16::from_le_bytes([m[0x70], m[0x71]]));
            let mut links = p.to_vec();
            let r = crate::orders::prune_chain(
                &mut links,
                &mut head,
                |l, k| {
                    i32::from(i16::from_le_bytes([
                        l[9 + k as usize][0x92],
                        l[9 + k as usize][0x93],
                    ]))
                },
                |l, k, v| l[9 + k as usize][0x92..0x94].copy_from_slice(&(v as i16).to_le_bytes()),
                |_, k| {
                    let mut ask8 = || {
                        eight += 1;
                        a[3 + (eight - 1).min(11)] != 0
                    };
                    if !ask8() {
                        return false;
                    }
                    gath.push(format!("p{}+0x0,{o},{who},0", 9 + k));
                    if a[21 + (gath.len() - 1).min(5)] == 0 {
                        return false;
                    }
                    if !ask8() {
                        return false;
                    }
                    bc += 1;
                    a[15 + (bc - 1).min(5)] != 0
                },
            );
            p.clone_from_slice(&links);
            if i16::from_le_bytes([m[0x70], m[0x71]]) >= 0 {
                m[0x70..0x72].copy_from_slice(&(head as i16).to_le_bytes());
            }
            log.push(if gath.is_empty() {
                "-".into()
            } else {
                gath.join("/")
            });
            i64::from(r)
        };
        assert_eq!(got, want_eax, "check_gatherers — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "check_gatherers — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "check_gatherers — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "check_gatherers — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "check_gatherers — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Build::check_gatherers@0062f710: {n} rows agree");
}
