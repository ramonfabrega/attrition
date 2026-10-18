//! `Army::do_mustering@006f4260` under the emulator against the port (item 3166; tools/sweep_gen.py, obj with windows).

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

/// `Army::do_mustering@006f4260`, run by `tools/emu/sweep_army_do_mustering.py` under unicorn
/// (tools/emu): 152 random bytes of object, 0 argument(s), `p` windows in order: p0..p7 the place(s), in flag order; the global cell 0x00c09970 points at 16 random bytes; the global cell 0x00c0998c points at 16 random bytes; 56792 random bytes at 0x00e3a390; callee 0x006f87c0 (0 args) stubbed; callee 0x006ec000 (0 args) stubbed; callee 0x006f5cc0 (3 args) stubbed; the word at `c0+0` points at 16 random bytes less `0` (placed); the word at `c0+4` points at 16 random bytes less `0` (placed); the word at `c0+8` points at 16 random bytes less `0` (placed); the word at `c0+12` points at 16 random bytes less `0` (placed); the word at `c1+0` points at 16 random bytes less `0` (placed); the word at `c1+4` points at 16 random bytes less `0` (placed); the word at `c1+8` points at 16 random bytes less `0` (placed); the word at `c1+12` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 1 answers 0..5 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_do_mustering.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // c0/c1 the cities cells' pointer arrays (p0..p3 and p4..p7 their records), c2 the leaders table.
            let rd = |w: &[u8], o: usize| {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            };
            let rh = |w: &[u8], o: usize| {
                i16::from_le_bytes(w[o..o + 2].try_into().expect("two bytes")) as i32
            };
            let idx = rh(m, 0x94) as usize;
            let lo = idx * 0x6eec;
            let flags = rh(&c[2], lo + 0xa68 + 2 * rd(m, 8) as usize) & 0xffff;
            let lead0 = rd(&c[2], lo);
            let mut spot = "-".to_string();
            let mut diff = "-".to_string();
            let mut done = false;
            if a[0] == 0 {
                let k = rd(m, 0x20);
                if k >= 0 {
                    let rec = &p[4 * idx + k as usize];
                    if rec[4] & 1 != 0 {
                        spot = format!("m+0x0,{},{},1", rh(rec, 8), idx);
                        if a[2] != 0 {
                            m[4] |= 0x10;
                            done = true;
                        }
                    }
                }
                if !done {
                    if flags & 8 != 0 && rd(m, 0x14) > 7 && lead0 & 0x300 != 0 {
                        wr(m, 4, 0x40);
                    } else {
                        wr(m, 4, 2);
                    }
                }
            } else if rd(m, 0x24) == 0 {
                let mut low = false;
                if flags & 4 != 0 {
                    diff = format!("c2+{:#x}", lo);
                    low = a[1] < 3;
                }
                if low {
                    wr(m, 4, 0x20);
                } else if flags & 8 == 0 || lead0 & 0x300 == 0 {
                    wr(m, 4, 2);
                } else {
                    wr(m, 4, 0x40);
                }
            } else {
                wr(m, 4, 2);
            }
            if !done {
                wr(m, 0x20, -1);
                wr(m, 0x38, rd(m, 0x48).wrapping_mul(0x300).wrapping_add(0x180));
                wr(m, 0x3c, rd(m, 0x4c).wrapping_mul(0x300).wrapping_add(0x180));
                wr(m, 0x40, rd(m, 0x50));
            }
            log.push(spot);
            log.push(diff);
            1
        };
        assert_eq!(got, want_eax, "do_mustering — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "do_mustering — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_mustering — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "do_mustering — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "do_mustering — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Army::do_mustering@006f4260: {n} rows agree");
}
