//! `Objects::find_free@0065ad60` under the emulator against the port (item 2400; tools/sweep_gen.py, obj with windows).

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

/// `Objects::find_free@0065ad60`, run by `tools/emu/sweep_objects_find_free.py` under unicorn
/// (tools/emu): 64 random bytes of object, 5 argument(s), argument 3 names 4 random bytes; vtable slot +0x18 (0 args) stubbed; the word at `m+0x14` points at 36 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); the word at `p1+8` points at 64 random bytes less `0` (placed); the word at `p1+12` points at 64 random bytes less `0` (placed); the word at `p1+16` points at 64 random bytes less `0` (placed); the word at `p1+20` points at 64 random bytes less `0` (placed); the word at `p1+24` points at 64 random bytes less `0` (placed); the word at `p1+28` points at 64 random bytes less `0` (placed); the word at `p1+32` points at 64 random bytes less `0` (placed); the word at `m+0x30` points at 36 random bytes less `0` (placed); the word at `p11+0` points at 64 random bytes less `0` (placed); the word at `p11+4` points at 64 random bytes less `0` (placed); the word at `p11+8` points at 64 random bytes less `0` (placed); the word at `p11+12` points at 64 random bytes less `0` (placed); the word at `p11+16` points at 64 random bytes less `0` (placed); the word at `p11+20` points at 64 random bytes less `0` (placed); the word at `p11+24` points at 64 random bytes less `0` (placed); the word at `p11+28` points at 64 random bytes less `0` (placed); the word at `p11+32` points at 64 random bytes less `0` (placed); the word at `0xc0aec0` points at 36 random bytes less `0` (placed); the word at `p21+0` points at 144 random bytes less `0` (placed); the word at `p21+4` points at 144 random bytes less `0` (placed); the word at `p21+8` points at 144 random bytes less `0` (placed); the word at `p21+12` points at 144 random bytes less `0` (placed); the word at `p21+16` points at 144 random bytes less `0` (placed); the word at `p21+20` points at 144 random bytes less `0` (placed); the word at `p21+24` points at 144 random bytes less `0` (placed); the word at `p21+28` points at 144 random bytes less `0` (placed); the word at `p21+32` points at 144 random bytes less `0` (placed); the word at `0xc0aedc` points at 36 random bytes less `0` (placed); the word at `p31+0` points at 144 random bytes less `0` (placed); the word at `p31+4` points at 144 random bytes less `0` (placed); the word at `p31+8` points at 144 random bytes less `0` (placed); the word at `p31+12` points at 144 random bytes less `0` (placed); the word at `p31+16` points at 144 random bytes less `0` (placed); the word at `p31+20` points at 144 random bytes less `0` (placed); the word at `p31+24` points at 144 random bytes less `0` (placed); the word at `p31+28` points at 144 random bytes less `0` (placed); the word at `p31+32` points at 144 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); the stubbed vtable laid at `p2+0`; stub 0 answers 9 values in call order, printed after the answers; the global cell 0x00c0618c holds `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_find_free.py") else {
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
            // `a`: who, base, limit, the mark's address, -1, then the vslot +0x18 stub's answer and
            // its nine in call order. p1/p11 (`this+0x14`/`+0x30`) are the objects' pointer
            // arrays of players 0/1, whose objects are p2..p10 / p12..p20; p21/p31 the units'
            // records' (`UnitData`, `+0x8e` the captain word), p22..p30 / p32..p40; p0 the mark.
            let who = a[0] as usize;
            let (obj0, rec0) = (2 + 10 * who, 22 + 10 * who);
            let rd16 = |w: &[u8], off: usize| i16::from_le_bytes([w[off], w[off + 1]]);
            let mut turn = 0;
            let mut mark = rd(&p[0], 0);
            let r = crate::find_free_steps(a[1], a[2], &mut mark, |i| {
                let i = i as usize;
                let alive = p[obj0 + i][8] & 1 != 0;
                let hold = rd16(&p[obj0 + i], 0x32);
                // The vslot is asked only of a dead number nothing holds.
                let is_unit = !alive && hold == 0 && {
                    turn += 1;
                    a[6 + turn - 1] != 0
                };
                crate::find_free_reusable(alive, hold, is_unit, rd16(&p[rec0 + i], 0x8e) < 0)
            });
            wr(&mut p[0], 0, mark);
            i64::from(r)
        };
        assert_eq!(got, want_eax, "find_free — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_free — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_free — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_free — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Objects::find_free@0065ad60: {n} rows agree");
}
