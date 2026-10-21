//! `Regions::find@00680180` under the emulator against the port (item 3397; tools/sweep_gen.py, obj with windows).

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

/// `Regions::find@00680180`, run by `tools/emu/sweep_regions_find.py` under unicorn
/// (tools/emu): 64 random bytes of object, 3 argument(s), `p` windows in order: p0..p4 the place(s), in flag order; callee 0x00538810 (0 args) stubbed; callee 0x00a1d660 (1 args) stubbed; callee 0x00a2e550 (6 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; the word at `0xc06188` points at 320 random bytes less `0` (placed); the word at `0xcae5fc` points at 4112 random bytes less `-0x10` (placed); the word at `m+0x10` points at 544 random bytes less `0` (placed); the word at `m+0x30` points at 1280 random bytes less `0` (placed); the word at `p0+0x134` points at 2800 random bytes less `0` (placed); stub 2 answered from [0] (90%); stub 2 answers 0..0 every row; the stack 1024 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_regions_find.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            // No crate method is `Regions::find` (the crate's region fill is `WorldData`'s, over its own
            // grids), so the breadth-first fill is ported inline over the windows: p0 the world (width,
            // height, queue capacity at +0/+4/+8), p1 the `div_3_table` (pointer at buffer + 0x10), p2
            // the region rows (0x88 apiece, a visit count at +0x14), p3 the queue of (x, y) pairs, p4 the
            // 0x1c-byte world cells (flags short, kind byte at +2, region short at +4).
            const MOVE_X: [i32; 9] = [0, -1, 0, 1, 1, 1, 0, -1, -1];
            const MOVE_Y: [i32; 9] = [0, -1, -1, -1, 0, 1, 1, 1, 0];
            let rd16 = |b: &[u8], o: usize| u16::from_le_bytes([b[o], b[o + 1]]);
            let (x0, y0, reg) = (a[0], a[1], a[2]);
            let (w, h, cap) = (rd(&p[0], 0), rd(&p[0], 4), rd(&p[0], 8));
            let cell = |x: i32, y: i32| (w.wrapping_mul(y).wrapping_add(x) as usize).wrapping_mul(0x1c);
            let wet = |p4: &[u8], at: usize| rd16(p4, at) & 0x100 == 0 && matches!(p4[at + 2] as i8, 1 | 2);
            wr(&mut p[2], (reg as usize) * 0x88 + 0x14, 0);
            wr(&mut p[3], 0, x0);
            wr(&mut p[3], 4, y0);
            let at0 = cell(x0, y0);
            p[4][at0 + 4..at0 + 6].copy_from_slice(&(reg as i16).to_le_bytes());
            let want = wet(&p[4], at0);
            let (mut out, mut inn) = (0i32, 1i32);
            loop {
                let cx = rd(&p[3], out as usize * 8);
                let cy = rd(&p[3], out as usize * 8 + 4);
                out = (out + 1) % (cap + 1);
                let n = rd(&p[2], (reg as usize) * 0x88 + 0x14);
                wr(&mut p[2], (reg as usize) * 0x88 + 0x14, n.wrapping_add(1));
                for k in 1..9 {
                    let ix = (MOVE_X[k] + cx) * 3 + 1;
                    let nx = rd(&p[1], (16 + 4 * ix as isize) as usize);
                    let ny = MOVE_Y[k] + cy;
                    if nx < 0 || ny < 0 || nx >= w || ny >= h {
                        continue;
                    }
                    let at = cell(nx, ny);
                    if wet(&p[4], at) == want && rd16(&p[4], at + 4) == 0 {
                        p[4][at + 4..at + 6].copy_from_slice(&(reg as i16).to_le_bytes());
                        wr(&mut p[3], inn as usize * 8, nx);
                        wr(&mut p[3], inn as usize * 8 + 4, ny);
                        inn = (inn + 1) % (cap + 1);
                    }
                }
                if out == 0 || inn == out {
                    break;
                }
            }
            0
            // end hand
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2500, "only {n} rows");
    eprintln!("Regions::find@00680180: {n} rows agree");
}
