//! `Border::process@0087fe30` under the emulator against the port (item 2946; tools/sweep_gen.py, obj with windows).

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

/// `Border::process@0087fe30`, run by `tools/emu/sweep_border_process.py` under unicorn
/// (tools/emu): 208 random bytes of object, 0 argument(s), 4 random bytes at 0x00cbe5ac; callee 0x008804a0 (1 args) stubbed; callee 0x00880c30 (0 args) stubbed; callee 0x00880960 (0 args) stubbed; callee 0x0087fbb0 (0 args) stubbed; callee 0x0087fd70 (0 args) stubbed; callee 0x0087fd00 (0 args) stubbed; callee 0x0087fcb0 (1 args) stubbed; the word at `m+0x4c` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 96 random bytes less `0` (placed); the word at `p0+4` points at 96 random bytes less `0` (placed); the word at `p0+8` points at 96 random bytes less `0` (placed); the word at `p0+12` points at 96 random bytes less `0` (placed); the word at `p0+16` points at 96 random bytes less `0` (placed); the word at `p0+20` points at 96 random bytes less `0` (placed); the word at `p0+24` points at 96 random bytes less `0` (placed); the word at `p0+28` points at 96 random bytes less `0` (placed); the word at `0xc0621c` points at 512 random bytes less `0` (placed); the word at `0xc0620c` points at 576 random bytes less `0` (placed); the word at `0xc06200` points at 672 random bytes less `0` (placed); the word at `0xc061e8` points at 2144 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 prints the 2 bytes at the cell 0x00cbe5ac when it is called, hex, after the record (`/` a call); stub 0 writes 4 (4 bytes) at `m+0x40` each call; stub 5 leaves `ecx` as a callee would, a value from [0, 7, -1, 12345], after the other numbers on the row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_border_process.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse::<i64>().expect("an integer") as i32).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Border::process: a rebuild is wanted when the border's own flag, the borders' flag or the scene's flag is
            // set; then, with changed players queued, the morph strips are built twice (the change state 0x40, then 0x80 --
            // the second a graphic tick on, if the first added strips) and the strips added get bit 2; the border is closed,
            // sorted, walked, and the visible list rebuilt. A camera change alone rebuilds the visible list. The morph and the
            // border are then processed and drawn (handed the process's stale ecx).
            log.clear();
            let tick = rd(&p[12], 0x844);
            let mut reads: Vec<String> = Vec::new();
            let mut build = |m: &mut [u8], c: &mut [Vec<u8>], state: u16, tick: i32, log: &mut Vec<String>| {
                c[0][0..2].copy_from_slice(&state.to_le_bytes());
                reads.push(format!("{:02x}{:02x}", state as u8, (state >> 8) as u8));
                log.push(format!("s0:m+0x0,{tick}"));
                wr(m, 0x40, 4);
            };
            let rebuild_wanted = m[0xc8] != 0 || p[9][0x161] != 0 || p[10][0x22a] != 0;
            let mut visible = false;
            if rebuild_wanted {
                if rd(m, 0x5c) != 0 {
                    let ebx = rd(m, 0x40);
                    build(m, c, 0x40, tick, &mut log);
                    let mut edi = rd(m, 0x40);
                    let t2 = if edi > ebx { tick + 2 } else { tick };
                    build(m, c, 0x80, t2, &mut log);
                    if rd(m, 0x40) == edi && edi > ebx {
                        while edi > ebx {
                            edi -= 1;
                            p[1 + edi as usize][0x4c] |= 4;
                        }
                    }
                    wr(m, 0x5c, 0);
                }
                log.extend(["s1:m+0x0", "s2:m+0x0", "s3:m+0x0"].map(String::from));
                m[0xc8] = 0;
                visible = true;
            } else if rd(&p[11], 0x28c) != 0 {
                visible = true;
            }
            if visible {
                log.push("s4:m+0x0".to_string());
            }
            log.push("s5:m+0x0".to_string());
            log.push(format!("s6:m+0x0,{}", a[7]));
            log.push(if reads.is_empty() { "-".to_string() } else { reads.join("/") });
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "process — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Border::process@0087fe30: {n} rows agree");
}
