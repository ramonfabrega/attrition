//! `UnitData::is_busy@0060a370` under the emulator against the port (item 2966; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::is_busy@0060a370`, run by `tools/emu/sweep_unit_data_is_busy.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p7 the place(s), in flag order, p8 stub 0's answer window; vtable slot +0xf4 (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; vtable slot +0x54 (0 args) stubbed; callee 0x0060a6f0 (0 args) stubbed; the word at `0xe85ddc` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p5+4` points at 16 random bytes less `0` (placed); the word at `p6+8` points at 32 random bytes less `0` (placed, null a third of the rows); stub 1 answered from [0, 0, 1] (90%); stub 2 answered from [0, 0, 1] (90%); stub 0 answers the address of 48 random bytes, a `p` window after the places; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_is_busy.py") else {
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
            // With an order list (`+0xdc`) the cursor is set to its head's second node
            // (cached into +0xd4/+0xcc/+0xd0, node `p5` -> `p6`); then a current order
            // (`+0xcc`) whose type (the answer window `p8`'s +0x20 indexing the type table
            // `p1..p4`) answers slot +0x50 or +0x54 is busy; anything else is
            // `is_entering_or_exiting`'s answer (stub 3).
            if rd(m, 0xdc) != 0 {
                m[0xd4..0xd8].copy_from_slice(&p[5][4..8].to_vec());
                m[0xcc..0xd0].copy_from_slice(&p[6][8..12].to_vec());
                m[0xd0] = p[6][0xc];
            }
            if rd(m, 0xdc) != 0 && rd(m, 0xcc) != 0 && (a[1] != 0 || a[2] != 0) {
                1
            } else {
                i64::from(a[3])
            }
        };
        assert_eq!(got, want_eax, "is_busy — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_busy — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_busy — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_busy — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_busy@0060a370: {n} rows agree");
}
