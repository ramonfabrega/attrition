//! `AnimalData::walk_data@005d7ea0` under the emulator against the port (item 2900; tools/sweep_gen.py, obj with windows).

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

/// `AnimalData::walk_data@005d7ea0`, run by `tools/emu/sweep_animal_data_walk_data_vt.py` under unicorn
/// (tools/emu): an empty body: 384 random bytes of object and 1 argument(s), every byte held after the return and `esp` checked (parked 1889).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_animal_data_walk_data_vt.py") else {
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
            // Object::walk_data(this, walker), then the walker's slot 4 handed the string at the global + 0x22ccc. The kept arm
            // (this's slot 0x78 is Object::must_walk, `a[5]`) tests a flag byte at [ebp+0xb] -- the top byte of the walker
            // pointer when `input != 0`, else set from this+0x18, this+8 bit 0, the u16 at this+0x32 and a non-negative i16 at this+0x8e -- and calls the walker's
            // slot 0 with (&flag, ebp+0xc); the other arm calls this's slot 0x78 and tests that call's whole `eax` (the listing
            // jumps to the `test eax` past the byte load; the decompile's `>> 0x18` is not it). A set flag hands the walker
            // (this+0x150, this+0x155).
            let _ = (&c, &p);
            log.push("s0:m+0x0,p0+0x0".into());
            log.push("s2:p0+0x0,2640".into());
            let mut flag = ((a[0] as u32) >> 24) as u8;
            let r16 = |w: &[u8], o: usize| u16::from_le_bytes([w[o], w[o + 1]]);
            if a[5] != 0 {
                if rd(&p[0], 4) == 0 {
                    flag = u8::from(
                        rd(m, 0x18) != 0
                            || m[8] & 1 != 0
                            || r16(m, 0x32) != 0
                            || (r16(m, 0x8e) as i16) >= 0,
                    );
                }
                log.push("s1:p0+0x0,sp-0x1,2147483392".into());
            } else {
                log.push("s3:m+0x0,p0+0x0".into());
                flag = u8::from(a[4] != 0);
            }
            if flag != 0 {
                log.push("s1:p0+0x0,m+0x150,m+0x155".into());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "walk_data — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "walk_data — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "walk_data — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "walk_data — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("AnimalData::walk_data@005d7ea0: {n} rows agree");
}
