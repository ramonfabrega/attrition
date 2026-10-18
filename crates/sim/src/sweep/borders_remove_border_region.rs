//! `Borders::remove_border_region@008836d0` under the emulator against the port (item 3049; tools/sweep_gen.py, obj with windows).

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

/// `Borders::remove_border_region@008836d0`, run by `tools/emu/sweep_borders_remove_border_region.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0..p9 the place(s), in flag order; the global cell 0x00c0621c points at 32 random bytes; the global cell 0x00c061b8 points at 32 random bytes; callee 0x00880c30 (0 args) stubbed; callee 0x0055d063 (4 args) stubbed; callee 0x0055d055 (2 args) stubbed; the word at `c0+0x10` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 212 random bytes less `-4` (placed); the word at `p0+4` points at 212 random bytes less `-4` (placed); the word at `p0+8` points at 212 random bytes less `-4` (placed); the word at `p0+12` points at 212 random bytes less `-4` (placed); the word at `p0+16` points at 212 random bytes less `-4` (placed); the word at `p0+20` points at 212 random bytes less `-4` (placed); the word at `p0+24` points at 212 random bytes less `-4` (placed); the word at `p0+28` points at 212 random bytes less `-4` (placed); the word at `c1+0x10` points at 1088 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 is `__cdecl`: the caller pops its arguments; stub 2 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_borders_remove_border_region.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Border number a[0] (1-based; 0 does nothing) of the borders cell `c0` (count +4, array +0x10 of
            // record pointers, p0; each record a `new[]` block of 0xcc-byte elements with its count at -4,
            // p1.., the pointer buffer+4): the record is closed (stub 0), destroyed (stub 1, the vector
            // destructor: pointer, 0xcc, count, the element destructor) and freed (stub 2: the block, count *
            // 0xcc + 4); then, if it was not the last, the last record takes its slot and the region table
            // (`c1`, array +0x10 -> p9, 0x88 a region, byte +0x30) is told the slot's number + 1 (low byte)
            // for the region (+0xc4 of the moved record); the count drops by one.
            if a[0] == 0 {
                for _ in 0..3 {
                    log.push("-".to_string());
                }
            } else {
                let idx = (a[0] - 1) as usize;
                let tok = format!("p{}", 1 + idx);
                let count = rd(&p[1 + idx], 0);
                log.push(format!("{tok}+0x4"));
                log.push(format!("{tok}+0x4,{tok}+0x4,204,{count},4850704"));
                log.push(format!("{tok}+0x4,{tok}+0x0,{}", count * 204 + 4));
                let n = rd(&c[0], 4);
                if (idx as i32) < n - 1 {
                    let last = rd(&p[0], (n as usize - 1) * 4);
                    wr(&mut p[0], idx * 4, last);
                    let region = rd(&p[n as usize], 0xc8) as usize;
                    let at = region * 0x88 + 0x30;
                    p[9][at] = (a[0] & 0xff) as u8;
                }
                wr(&mut c[0], 4, n - 1);
            }
            want_eax
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "remove_border_region — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "remove_border_region — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "remove_border_region — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "remove_border_region — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Borders::remove_border_region@008836d0: {n} rows agree");
}
