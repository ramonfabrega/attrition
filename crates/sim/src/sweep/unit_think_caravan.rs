//! `Unit::think_caravan@005f5650` under the emulator against the port (item 2946; tools/sweep_gen.py, obj with windows).

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

/// `Unit::think_caravan@005f5650`, run by `tools/emu/sweep_unit_think_caravan.py` under unicorn
/// (tools/emu): 180 random bytes of object, 1 argument(s), callee 0x0065ba90 (9 args) stubbed; callee 0x005e4dc0 (6 args) stubbed; the word at `0xc0618c` points at 528 random bytes less `0` (placed); the word at `0xc09970` points at 16 random bytes less `0` (placed); the word at `0xc0998c` points at 16 random bytes less `0` (placed); the word at `0xc099a8` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 128 random bytes less `0` (placed); the word at `p1+4` points at 128 random bytes less `0` (placed); the word at `p1+8` points at 128 random bytes less `0` (placed); the word at `p1+12` points at 128 random bytes less `0` (placed); the word at `p2+0` points at 128 random bytes less `0` (placed); the word at `p2+4` points at 128 random bytes less `0` (placed); the word at `p2+8` points at 128 random bytes less `0` (placed); the word at `p2+12` points at 128 random bytes less `0` (placed); the word at `p3+0` points at 128 random bytes less `0` (placed); the word at `p3+4` points at 128 random bytes less `0` (placed); the word at `p3+8` points at 128 random bytes less `0` (placed); the word at `p3+12` points at 128 random bytes less `0` (placed); the word at `0xc061b4` points at 128 random bytes less `0` (placed); stub 0 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_think_caravan.py") else {
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
            // think_caravan: unless forced (`param_1`), the caravan acts when its level is the leader option's threshold (1
            // when flag 0x40000 is set; else 7, 12, 17, 32, 62 for the option 1..5, 2 otherwise) or five past a multiple; it
            // asks find_city for the nearest allied city that can trade and, if it is live, queues a trade order to it.
            log.extend(["-"; 2].map(String::from));
            let owner = m[9] as usize;
            'g: {
                if a[0] == 0 {
                    let thr: i32 = if rd(m, 0x68) & 0x40000 != 0 {
                        1
                    } else {
                        match rd(&p[16], 8 + 32 * owner) {
                            1 => 7,
                            2 => 0xc,
                            3 => 0x11,
                            4 => 0x20,
                            5 => 0x3e,
                            _ => 2,
                        }
                    };
                    let lvl = m[0xb0] as i32;
                    if lvl < thr || (lvl != thr && (lvl - 2) % 5 != 0) {
                        break 'g 0;
                    }
                }
                let y = rd(m, 0x14) ^ 0x63637;
                let x = rd(m, 0x10) ^ 0x63637;
                log[0] = format!("{y},{x},{y},4,{owner},{y},0,18,0,0");
                let idx = a[1];
                if idx < 0 {
                    break 'g 0;
                }
                let rec = &p[4 + 4 * rd(&p[0], 0x200) as usize + idx as usize];
                if rec[4] & 1 == 0 {
                    break 'g 0;
                }
                let short = i16::from_le_bytes([rec[8], rec[9]]);
                log[1] = format!("m+0x0,{short},{},-1,-1,2,0", rec[0x5e] as i8);
                1
            }
        };
        assert_eq!(got, want_eax, "think_caravan — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "think_caravan — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "think_caravan — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "think_caravan — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "think_caravan — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::think_caravan@005f5650: {n} rows agree");
}
