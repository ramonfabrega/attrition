//! `Unit::replace_form_id@005fd420` under the emulator against the port (item 2863; tools/sweep_gen.py, obj with windows).

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

/// `Unit::replace_form_id@005fd420`, run by `tools/emu/sweep_unit_replace_form_id.py` under unicorn
/// (tools/emu): 272 random bytes of object, 1 argument(s), vtable slot +0x94 (0 args) stubbed; vtable slot +0x10c (0 args) stubbed; the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p1+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p2+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `m+0x100` points at 32 random bytes less `0` (placed); the word at `m+0x104` points at 32 random bytes less `0` (placed); the word at `m+0x108` points at 32 random bytes less `0` (placed); stub 1 answered from [1, 1, 0] (90%); the word at `p2+0` laid as the address of `p0!` half the rows; stub 0 answers `p6+j`, the data of the object its `ecx` is in `p3+j` (j < 3), linked at `+0x1a`; stub 1 answers 3 values in call order, printed after the answers; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_replace_form_id.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
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
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The list head `m+0xdc` (`p[0]`) starts the walk (node, object, byte into `m+0xd4`, `+0xcc`, `+0xd0`);
                // when `m+0xd8` is set each node's order object (`p[3 + k]`, null leaves the walk) answering slot
                // +0x10c (`a[3..6]` in call order) has its slot +0x94 record (`p[6 + k]`) take the argument at +0x10.
                // The nodes are a ring of three, `p[0]`, `p[1]`, `p[2]`, the last pointing back at the head.
                if rd(m, 0xdc) != 0 {
                    m.copy_within(0xdc..0xe0, 0xd4);
                    m[0xcc..0xd0].copy_from_slice(&p[0][8..12]);
                    m[0xd0] = p[0][0xc];
                }
                if rd(m, 0xd8) != 0 {
                    let (mut k, mut n) = (0usize, 0usize);
                    while rd(m, 0xcc) != 0 {
                        let ask = a[3 + n.min(2)] != 0;
                        n += 1;
                        if ask {
                            wr(&mut p[6 + k], 0x10, a[0]);
                        }
                        if rd(m, 0xdc) != 0 {
                            m[0xd4..0xd8].copy_from_slice(&p[k][0..4]);
                            k = (k + 1) % 3;
                            m[0xcc..0xd0].copy_from_slice(&p[k][8..12]);
                            m[0xd0] = p[k][0xc];
                        }
                        if rd(m, 0xd4) == rd(m, 0xdc) {
                            break;
                        }
                    }
                }
                0
            }
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "replace_form_id — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "replace_form_id — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "replace_form_id — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::replace_form_id@005fd420: {n} rows agree");
}
