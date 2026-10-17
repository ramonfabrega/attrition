//! `Unit::update_patrol_order@005e35e0` under the emulator against the port (item 2863; tools/sweep_gen.py, obj with windows).

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

/// `Unit::update_patrol_order@005e35e0`, run by `tools/emu/sweep_unit_update_patrol_order.py` under unicorn
/// (tools/emu): 272 random bytes of object, 1 argument(s), vtable slot +0x94 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0xa4 (0 args) stubbed; the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p2+4` points at 16 random bytes less `0` (placed); the word at `p3+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed); the word at `p3+8` points at 16 random bytes less `0` (placed); the word at `m+0x100` points at 64 random bytes less `0` (placed); the word at `m+0x104` points at 64 random bytes less `0` (placed); the word at `m+0x108` points at 64 random bytes less `0` (placed); the word at `p8+0` points at 16 random bytes less `0` (placed); the word at `p9+0` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 16 random bytes less `0` (placed); stub 1 answered from [22, 22, 22, 0] (90%); the word at `p1+0` laid as the address of `p0` half the rows; the word at `p2+0` laid as the address of `p0` half the rows; the word at `p3+0` laid as the address of `p0` half the rows; the word at `p4+0` laid as the address of `p0!` half the rows; stub 0 answers `p8+j`, the data of the object its `ecx` is in `p5+j` (j < 3), linked at `+0x3a`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0x20` as well as at `this`; the stubbed vtable laid at `p9+0x20` as well as at `this`; the stubbed vtable laid at `p10+0x20` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_update_patrol_order.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The patrol list: `m+0xdc` points at the list `p[0]`, whose +4 is the first node `p[1]`; node k
                // is `p[k]` (+0 its list, +4 the next, +8 its order object `p[4 + k]`, +0xc a byte), the fourth is
                // the end node, whose +0 is the list: the walk goes on while the next node's +0 is not the list. It keeps the node, its object and the byte in `m+0xd4`, `+0xcc`, `+0xd0`.
                // Each node's order object answers slot +0x94 with its record `p[7 + k]` (`a[1]` is that stub's
                // lone answer, unused), whose word 3 is compared with the argument (-1 takes any) and which, with
                // slot +0x10 (`a[2]`) answering 0x16, returns slot +0xa4's answer (`a[3]`).
                'walk: {
                    if rd(m, 0xdc) == 0 {
                        break 'walk 0;
                    }
                    m[0xd4..0xd8].copy_from_slice(&p[0][4..8]);
                    m[0xcc..0xd0].copy_from_slice(&p[1][8..12]);
                    m[0xd0] = p[1][0xc];
                    let mut k = 1usize;
                    loop {
                        if (a[0] == -1 || rd(&p[7 + k], 0xc) == a[0]) && a[2] == 0x16 {
                            break 'walk i64::from(a[3]);
                        }
                        m[0xd4..0xd8].copy_from_slice(&p[k][4..8]);
                        m[0xcc..0xd0].copy_from_slice(&p[k + 1][8..12]);
                        m[0xd0] = p[k + 1][0xc];
                        k += 1;
                        if rd(&p[k], 0) == rd(m, 0xdc) {
                            break 'walk 0;
                        }
                    }
                }
            }
        };
        assert_eq!(got, want_eax, "update_patrol_order — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_patrol_order — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "update_patrol_order — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "update_patrol_order — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::update_patrol_order@005e35e0: {n} rows agree");
}
