//! `GroupData::group_air@0070b160` under the emulator against the port (item 2797; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::group_air@0070b160`, run by `tools/emu/sweep_group_data_group_air.py` under unicorn
/// (tools/emu): 2508 random bytes of object, 2 argument(s), argument 0 names 16 random bytes; vtable slot +0xc (4 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 68 random bytes less `0` (placed); the word at `p1+4` points at 68 random bytes less `0` (placed); the word at `p1+8` points at 68 random bytes less `0` (placed); the word at `p1+12` points at 68 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 68 random bytes less `0` (placed); the word at `p6+4` points at 68 random bytes less `0` (placed); the word at `p6+8` points at 68 random bytes less `0` (placed); the word at `p6+12` points at 68 random bytes less `0` (placed); the word at `p2+0x18` points at 544 random bytes less `0` (placed); the word at `p3+0x18` points at 544 random bytes less `0` (placed); the word at `p4+0x18` points at 544 random bytes less `0` (placed); the word at `p5+0x18` points at 544 random bytes less `0` (placed); the word at `p7+0x18` points at 544 random bytes less `0` (placed); the word at `p8+0x18` points at 544 random bytes less `0` (placed); the word at `p9+0x18` points at 544 random bytes less `0` (placed); the word at `p10+0x18` points at 544 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 1 answers 4 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; figures p2..p5 link at +0x28 (2 bytes) above their own index or to -1; figures p7..p10 link at +0x28 (2 bytes) below their own index or to -1; the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_group_air.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Each listed object (`objects[who][list[k]]`) opens its passenger chain (+0x28 index, +0x3e table, -1
                // ends); a passenger whose type record (+0x18, its +0x218 word) is 2 reaches the group's slot +0xc with
                // `(index, table, 0, 0)` when `type_arg` is TYPE_NONE (-1) or `is(type_arg, 0)` answers non-zero: slot
                // +0xb8 is `ObjectData::is` (a[13], the type's +0x60, stub 1 a[5..9]) or its own stub (stub 2 a[9..13]).
                let num = rd(m, 12);
                let who = usize::from(m[0x4a]);
                let rec = |w: usize, i: usize| &p[2 + 5 * w + i];
                let ty = |w: usize, i: usize| &p[11 + 4 * w + i];
                let link = |o: &[u8]| (i16::from_le_bytes([o[0x28], o[0x29]]), o[0x3e] as i8);
                let (mut n_is, mut n_own) = (0usize, 0usize);
                let mut calls: Vec<String> = Vec::new();
                for k in 0..usize::try_from(num.max(0)).expect("a count") {
                    let i = usize::try_from(i16::from_le_bytes([m[0x8cc + 2 * k], m[0x8cd + 2 * k]]))
                        .expect("an index");
                    let (mut nx, mut nw) = link(rec(who, i));
                    while nx >= 0 && nw >= 0 {
                        let (w2, i2) = (
                            usize::try_from(nw).expect("a table"),
                            usize::try_from(nx).expect("an index"),
                        );
                        if rd(ty(w2, i2), 0x218) == 2 {
                            let pass = a[1] == -1 || {
                                if a[13] != 0 {
                                    n_is += 1;
                                    a[5 + (n_is - 1).min(3)] != 0
                                } else {
                                    n_own += 1;
                                    a[9 + (n_own - 1).min(3)] != 0
                                }
                            };
                            if pass {
                                calls.push(format!("p0+0x0,{nx},{nw},0,0"));
                            }
                        }
                        (nx, nw) = link(rec(w2, i2));
                    }
                }
                log.push(if calls.is_empty() {
                    "-".into()
                } else {
                    calls.join("/")
                });
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "group_air — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "group_air — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "group_air — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "group_air — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::group_air@0070b160: {n} rows agree");
}
