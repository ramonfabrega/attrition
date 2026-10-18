//! `UnitData::find_def_pos@00608560` under the emulator against the port (item 3166; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::find_def_pos@00608560`, run by `tools/emu/sweep_unit_data_find_def_pos.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 pointed argument(s), p2..p4 the place(s), in flag order, p5..p8 the stubs' answer window(s); the global cell 0x00cae5fc points at 1024 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x00608450 (0 args) stubbed; callee 0x0060b030 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0xb8 (0 args) stubbed; the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p2+4` points at 16 random bytes less `0` (placed); the word at `p3+8` points at 16 random bytes less `0` (placed); stub 2 answered from [10, 10, 0, 3] (90%); stub 4 answered from [0, 1] (90%); stub 0 answers the address of 32 random bytes, a `p` window after the places; stub 1 answers the address of 32 random bytes, a `p` window after the places; stub 3 answers the address of 32 random bytes, a `p` window after the places; stub 5 answers the address of 32 random bytes, a `p` window after the places; stub 2 answers 2 values in call order, the last from [0, 3], printed after the answers; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_find_def_pos.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // p0/p1 the two Coord out-words, p2 the list head, p3 its node, p4 the node's order; p5 get_action's
            // answer, p6 get_order's, p7 the record slot +0xcc answers, p8 the one slot +0xb8 answers; a[8], a[9] the
            // slot +0x10 answers in call order, a[6] slot +0x14's.
            let rdw = |w: &[u8], o: usize| {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            };
            let mut done = false;
            if a[8] == 10 && p[7][0x1d] != 0 {
                let (x, y) = (rdw(&p[7], 0x14), rdw(&p[7], 0x18));
                wr(&mut p[0], 0, x);
                wr(&mut p[1], 0, y);
                done = true;
            }
            if !done && rdw(m, 0xdc) != 0 {
                let node = rdw(&p[2], 4);
                wr(m, 0xd4, node);
                wr(m, 0xcc, rdw(&p[3], 8));
                m[0xd0] = p[3][0xc];
                if a[9] != 0 && p[4][4] & 8 != 0 && a[6] != 0 {
                    let (x, y) = (rdw(&p[8], 4), rdw(&p[8], 8));
                    wr(&mut p[0], 0, x);
                    wr(&mut p[1], 0, y);
                    done = true;
                }
            }
            if done {
                1
            } else {
                let ix = rdw(m, 0x10) ^ 0x63637;
                let iy = rdw(m, 0x14) ^ 0x63637;
                let x = rdw(&c[0], 4 * (ix >> 4) as usize) * 0x30 + 0x18;
                let y = rdw(&c[0], 4 * (iy >> 4) as usize) * 0x30 + 0x18;
                wr(&mut p[0], 0, x);
                wr(&mut p[1], 0, y);
                0
            }
        };
        assert_eq!(got, want_eax, "find_def_pos — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_def_pos — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_def_pos — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "find_def_pos — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::find_def_pos@00608560: {n} rows agree");
}
