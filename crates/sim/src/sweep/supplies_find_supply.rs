//! `Supplies::find_supply@0073aba0` under the emulator against the port (item 2966; tools/sweep_gen.py, obj with windows).

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

/// `Supplies::find_supply@0073aba0`, run by `tools/emu/sweep_supplies_find_supply.py` under unicorn
/// (tools/emu): 64 random bytes of object, 3 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p17 the place(s), in flag order; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x0073b560 (0 args) stubbed; callee 0x0046cff0 (0 args) stubbed; the word at `0xe3a1c0+a[2]*0x1c` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p2+4` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed); the word at `p2+12` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+a[2]*0x1c` points at 16 random bytes less `0` (placed); the word at `p7+0` points at 16 random bytes less `0` (placed); the word at `p7+4` points at 16 random bytes less `0` (placed); the word at `p7+8` points at 16 random bytes less `0` (placed); the word at `p7+12` points at 16 random bytes less `0` (placed); the word at `0xc0618c` points at 96 random bytes less `0` (placed); the word at `p12+0x14+a[2]*0x1c` points at 16 random bytes less `0` (placed); the word at `p13+0` points at 32 random bytes less `0` (placed); the word at `p13+4` points at 32 random bytes less `0` (placed); the word at `p13+8` points at 32 random bytes less `0` (placed); the word at `p13+12` points at 32 random bytes less `0` (placed); stub 0 answered from [1, 1, 0] (90%); stub 1 answered from [1, 1, 0] (90%); stub 2 answers 0..100 every row; stub 3 answers 0..25000 every row; stub 2 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x434:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_supplies_find_supply.py") else {
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
            // `a` is the point (x, y), the player, then the answers: slots +0x8 and +0xbc of
            // the supply's unit, `get_radius`, `vector_dist`. The player's supply count is
            // the word `p0`/`p1` (the leaders table's +0x434); its supply records `p3..p6`
            // (flag bit 0 at +4, the unit's index the short at +2) name units `p8..p11`
            // and objects `p14..p17`, whose coordinates (+0x10, +0x14, keyed 0x63637) are
            // measured from the point. A supply is found when its distance is within
            // radius * 192. `get_radius`'s leftover `edx` is the previous measure's.
            let player = a[2] as usize;
            let count = rd(&p[player], 0);
            let (mut r2, mut r3): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
            let mut stale = 0;
            let mut found = -1i64;
            for i in 0..count.max(0) as usize {
                let rec = &p[3 + i];
                if rec[4] & 1 == 0 || a[3] == 0 || a[4] == 0 {
                    continue;
                }
                let idx = usize::from(i16::from_le_bytes([rec[2], rec[3]]) as u16);
                r2.push(format!("p{}+0x0,{stale}", 3 + i));
                let obj = &p[14 + idx];
                let ecx = a[0].wrapping_sub(rd(obj, 0x10) ^ 0x63637).wrapping_abs();
                let edx = a[1].wrapping_sub(rd(obj, 0x14) ^ 0x63637).wrapping_abs();
                stale = edx;
                r3.push(format!("{ecx},{edx}"));
                if a[6] <= a[5].wrapping_mul(0xc0) {
                    found = i as i64;
                    break;
                }
            }
            let col = |v: &[String]| if v.is_empty() { "-".to_string() } else { v.join("/") };
            log.push(col(&r2));
            log.push(col(&r3));
            found
        };
        assert_eq!(got, want_eax, "find_supply — eax, row `{line}`");
        assert_eq!(log, want_log, "find_supply — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_supply — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_supply — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_supply — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Supplies::find_supply@0073aba0: {n} rows agree");
}
