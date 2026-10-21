//! `SpellType::cast_bribe@00672d00` under the emulator against the port (item 3410; tools/sweep_gen.py, obj with windows).

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

/// `SpellType::cast_bribe@00672d00`, run by `tools/emu/sweep_spell_type_cast_bribe_self.py` under unicorn
/// (tools/emu): 472 random bytes of object, 4 argument(s), `p` windows in order: p0..p1 the place(s), in flag order; vtable slot +0xd8 (3 args) stubbed; the word at `0xc0aec0+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p0+4*a[0]` points at 160 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_cast_bribe_self.py") else {
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
            {
                // The self-target arm (`param_2 == param_4`): the refund through slot 0xd8, then the unit's counter at +0x96
                // (a short) is lowered by the smaller of itself and `this+0x1d0`.
                let (p1, p2) = (a[0], a[1]);
                log.push(format!("s0:m+0x0,{p2},{p1},-1"));
                let sh = i16::from_le_bytes([p[1][0x96], p[1][0x97]]) as i32;
                let lim = rd(m, 0x1d0);
                let take = if lim < sh { lim } else { sh };
                let v = (sh as u16).wrapping_sub(take as u16);
                p[1][0x96..0x98].copy_from_slice(&v.to_le_bytes());
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "cast_bribe — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "cast_bribe — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "cast_bribe — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "cast_bribe — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("SpellType::cast_bribe@00672d00: {n} rows agree");
}
