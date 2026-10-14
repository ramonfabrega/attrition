//! `Unit::update_action@0060a870` under the emulator against the port (item 2341; tools/sweep_gen.py, obj with windows).

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

/// `Unit::update_action@0060a870`, run by `tools/emu/sweep_unit_update_action.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), 16 random bytes at 0x00e90000; 16 random bytes at 0x00e90010; 16 random bytes at 0x00e90020; 16 random bytes at 0x00e90030; 32 random bytes at 0x00e90100; 32 random bytes at 0x00e90120; 32 random bytes at 0x00e90140; 32 random bytes at 0x00e90160; vtable slot +0x10 (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; the stubbed vtable laid at `c4+0`; the stubbed vtable laid at `c5+0` as well as at `this`; the stubbed vtable laid at `c6+0` as well as at `this`; the stubbed vtable laid at `c7+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_update_action.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(8);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The nodes `c[0..4]` sit at `NODES + 0x10 k` (`--at`), node k's
            // order `c[4 + k]`; the list is read off the bytes as the port's
            // queue: `head_node->next` first, up to `head_node` itself
            // (`+0xdc`). Each order answers from its own window (the
            // script's hand edit): `is_move` +8, `get_type` +0xc, the move
            // record `+0x10`.
            const NODES: i32 = 0xe9_0000;
            let node = |addr: i32| ((addr - NODES) / 0x10) as usize;
            let head = rd(m, 0xdc);
            let mut seq = Vec::new();
            if head != 0 {
                let mut k = node(rd(&c[node(head)], 4));
                seq.push(k);
                while k != node(head) {
                    k = node(rd(&c[k], 4));
                    seq.push(k);
                }
            }
            let n = seq.len();
            let mut cx = (m, c, seq);
            let got = crate::orders::update_action_steps(
                &mut cx,
                n,
                |(m, _, _)| {
                    wr(m, 0x70, rd(m, 0x10) ^ 0x63637);
                    wr(m, 0x74, rd(m, 0x14) ^ 0x63637);
                    wr(m, 0x58, rd(m, 0x50));
                },
                |(m, c, seq), i| {
                    let k = seq[i];
                    wr(m, 0xd4, NODES + 0x10 * k as i32);
                    wr(m, 0xcc, rd(&c[k], 8));
                    m[0xd0] = c[k][0xc];
                },
                |(_, c, seq), i| rd(&c[4 + seq[i]], 8) != 0,
                |(_, c, seq), i| c[4 + seq[i]][4] & 4 != 0,
                |(_, c, seq), i| rd(&c[4 + seq[i]], 0xc) == 0x12,
                |(m, c, seq), i| {
                    let o = &c[4 + seq[i]];
                    wr(m, 0x70, rd(o, 0x14));
                    wr(m, 0x74, rd(o, 0x18));
                    wr(m, 0x58, rd(o, 0x1c));
                },
            );
            let (_, c, seq) = cx;
            got.map_or(0, |i| i64::from(rd(&c[seq[i]], 8)))
        };
        assert_eq!(got, want_eax, "update_action — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_action — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "update_action — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[9 + i],
                "update_action — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("Unit::update_action@0060a870: {n} rows agree");
}
