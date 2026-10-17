//! `Borders::save_last_who@008835e0` under the emulator against the port (item 3038; tools/sweep_gen.py, obj with windows).

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

/// `Borders::save_last_who@008835e0`, run by `tools/emu/sweep_borders_save_last_who.py` under unicorn
/// (tools/emu): 8 random bytes of object, 4 argument(s), `p` windows in order: p0..p5 the place(s), in flag order; the global cell 0x00c0621c points at 1024 random bytes; the global cell 0x00c061d0 points at 1024 random bytes; callee 0x00882300 (1 args) stubbed; callee 0x004a0cb0 (1 args) stubbed; the word at `c0+0x10` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `p0+4` points at 128 random bytes less `0` (placed); the word at `p0+8` points at 128 random bytes less `0` (placed); the word at `c0+0x2c` points at 512 random bytes less `0` (placed); the word at `c1+0x134` points at 1824 random bytes less `0` (placed); stub 0 answered from [0, 1, 1] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 prints the 2 bytes its argument 0 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_borders_save_last_who.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The borders cell's 8-byte cells (table at +0x2c, 16-bit index `width(+0x224) * y + x`) get
            // the two sides' marks in their flag byte (+1): 0x40 for the tile's own owner (the tile record's
            // signed byte +0xf, row-major in the world cell), 0x80 for `a[2]` (argument 2, who), when the
            // sides differ and `a[3]` is non-zero; both sides hidden (the stub, one answer for both calls)
            // is a return. A mark on an unmarked cell (flag byte & 0xc0 == 0) adds the index to the
            // `a[3]`-th border's list (stub 1, `ecx` = that record + 0x58).
            let (x, y, who, side) = (a[0], a[1], a[2], a[3]);
            let hidden = a[4] != 0;
            let width = i32::from(u16::from_le_bytes([c[0][0x224], c[0][0x225]]));
            let idx = ((width * y + x) as u16) as usize;
            let tile = ((rd(&c[1], 0) * y + x) * 0x1c) as usize;
            let esi = i32::from(p[5][tile + 0xf] as i8).max(-1);
            let edi = who.max(-1);
            let (mut adds, mut derefs) = (Vec::new(), Vec::new());
            'body: {
                if edi >= 0 && esi >= 0 && hidden {
                    break 'body;
                }
                let at = idx * 8 + 1;
                let old = p[4][at];
                let edx = old & 0xc0;
                if side == 0 || edi == esi {
                    break 'body;
                }
                let marked = esi >= 0;
                if marked {
                    p[4][at] = old | 0x40;
                }
                if edi >= 0 {
                    p[4][at] |= 0x80;
                } else if !marked {
                    break 'body;
                }
                if edx == 0 {
                    adds.push(format!("p{side}+0x58,sp-0x10"));
                    derefs.push(format!("{:02x}{:02x}", idx & 0xff, idx >> 8));
                }
            }
            log.push(if adds.is_empty() {
                "-".into()
            } else {
                adds.join("/")
            });
            log.push(if derefs.is_empty() {
                "-".into()
            } else {
                derefs.join("/")
            });
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "save_last_who — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "save_last_who — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "save_last_who — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "save_last_who — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 2500, "only {n} rows");
    eprintln!("Borders::save_last_who@008835e0: {n} rows agree");
}
