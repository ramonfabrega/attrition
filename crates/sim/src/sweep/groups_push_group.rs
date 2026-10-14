//! `Groups::push_group@0070f9e0` under the emulator against the port (item 2314; tools/sweep_gen.py, obj with windows).

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

/// `Groups::push_group@0070f9e0`, run by `tools/emu/sweep_groups_push_group.py` under unicorn
/// (tools/emu): 16 random bytes of object, 3 argument(s), the global cell 0x00e85f20 points at 10064 random bytes; 8 random bytes at 0x00e85f2c; argument 1 names 2516 random bytes; callee 0x00708000 (1 args) stubbed; callee 0x006fa460 (2 args) stubbed; callee 0x006fa690 (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0x10 (4 args) stubbed; the word at `0xc0618c` points at 256 random bytes less `0` (placed); the word at `p1+0x14+0x1c*a[0]` points at 32 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p2+4` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed); the word at `p2+12` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*a[0]` points at 32 random bytes less `0` (placed); the word at `p7+0` points at 132 random bytes less `0` (placed); the word at `p7+4` points at 132 random bytes less `0` (placed); the word at `p7+8` points at 132 random bytes less `0` (placed); the word at `p7+12` points at 132 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 3 answered from [0, 1, 1] (90%); stub 3 answers 8 values in call order, printed after the answers; stub 1 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `c0+0` as well as at `this`; the stubbed vtable laid at `c0+0x9d4` as well as at `this`; the stubbed vtable laid at `c0+0x13a8` as well as at `this`; the stubbed vtable laid at `c0+0x1d7c` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_groups_push_group.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a[0]` who, `a[2]` force; the answers `a[3]` (`equals_group`)
            // and `a[4]` (`get_open_slot`), `+8` (in use) `a[8..16]` in call
            // order. The group is `p[0]` (`+0xc` num, the shorts at
            // `+0x8cc`), member `o`'s unit `p[8 + o]` (`+0x80` its group,
            // `+0xa` o, `+9` who); `c[0]` the pool, `c[1]` `last_group`. The
            // log is `equals_group`'s `ecx,arg`, `copy_group`'s `ecx`, then
            // each kill's `ecx,o,who,0,0`.
            let rec = |s: i32| format!("c0+{:#x}", s * 0x9d4);
            let mm = p[0].to_vec();
            let member =
                |k: usize| i16::from_le_bytes([mm[0x8cc + 2 * k], mm[0x8cd + 2 * k]]) as usize;
            let w = 4 * a[0] as usize;
            let mut used = 0usize;
            let (mut eq, mut copy, mut kills) =
                ("-".to_string(), "-".to_string(), Vec::<String>::new());
            let s = crate::group::push_group_steps(
                p,
                rd(&mm, 0xc),
                a[2] != 0,
                |_, _| {
                    used += 1;
                    a[8 + (used - 1).min(7)] != 0
                },
                |_| {
                    let last = rd(&c[1], w);
                    eq = format!("p0+0x0,{}", rec(last));
                    if a[3] != 0 {
                        return last;
                    }
                    copy = rec(a[4]);
                    wr(&mut c[1], w, a[4]);
                    a[4]
                },
                |p, k| i16::from_le_bytes([p[8 + member(k)][0x80], p[8 + member(k)][0x81]]),
                |p, old, k| {
                    let u = &p[8 + member(k)];
                    let o = i16::from_le_bytes([u[0xa], u[0xb]]);
                    kills.push(format!("{},{o},{},0,0", rec(i32::from(old)), u[9]));
                },
                |p, k, slot| {
                    let u = &mut p[8 + member(k)];
                    u[0x80..0x82].copy_from_slice(&(slot as i16).to_le_bytes());
                },
            );
            log.push(eq);
            log.push(copy);
            log.push(if kills.is_empty() {
                "-".into()
            } else {
                kills.join("/")
            });
            i64::from(s)
        };
        assert_eq!(got, want_eax, "push_group — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "push_group — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "push_group — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "push_group — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "push_group — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Groups::push_group@0070f9e0: {n} rows agree");
}
