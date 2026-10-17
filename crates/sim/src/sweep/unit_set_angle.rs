//! `Unit::set_angle@00605400` under the emulator against the port (item 2844; tools/sweep_gen.py, obj with windows).

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

/// `Unit::set_angle@00605400`, run by `tools/emu/sweep_unit_set_angle.py` under unicorn
/// (tools/emu): 256 random bytes of object, 3 argument(s), callee 0x0070ccb0 (1 args) stubbed; callee 0x005d9010 (2 args) stubbed; the word at `m+0xf4` points at 8 random bytes less `0` (placed); the word at `p12+0` points at 8 random bytes less `0` (placed); stub 0 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 4 rows 0x9d4 apart at `0xe85f20`, slices +0xc:0x4, +0x48:0x4, +0x8cc:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_set_angle.py") else {
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
            // A turn of more than a quarter circle (the difference from `m+0x50`, as unsigned, in
            // 0x40000000..=0xc0000000) flips bit 1 of `m+0x68` and, for the group in the short at `m+0x80`
            // (row k of the groups table: `p[3k]` its count, `p[3k+1]` its +0x48 and +0x49 bytes, `p[3k+2]`
            // its first member), flips the byte at +0x48 when the group's leader (`find_leader`,
            // `a[3]`, for a unit group; the first member for a building group) is this unit's id (the
            // short at `m+0xa`); the angle is stored and `Guy::set_angle` handed it (item 2844).
            let angle = a[0];
            let d = (angle as u32).wrapping_sub(rd(m, 0x50) as u32);
            if (0x4000_0000..=0xc000_0000).contains(&d) {
                wr(m, 0x68, rd(m, 0x68) ^ 2);
                let g = i32::from(i16::from_le_bytes([m[0x80], m[0x81]]));
                if g >= 0 {
                    let k = g as usize;
                    let leader = if rd(&p[3 * k], 0) < 1 {
                        -1
                    } else if p[3 * k + 1][1] == 0 {
                        log.push(format!("s0:r{k}+0x0,0"));
                        a[3]
                    } else {
                        i32::from(i16::from_le_bytes([p[3 * k + 2][0], p[3 * k + 2][1]]))
                    };
                    if leader == i32::from(i16::from_le_bytes([m[0xa], m[0xb]])) {
                        p[3 * k + 1][0] = u8::from(p[3 * k + 1][0] == 0);
                    }
                }
            }
            wr(m, 0x50, angle);
            log.push(format!("s1:p13+0x0,{},{}", a[0], a[2]));
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "set_angle — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_angle — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_angle — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_angle — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::set_angle@00605400: {n} rows agree");
}
