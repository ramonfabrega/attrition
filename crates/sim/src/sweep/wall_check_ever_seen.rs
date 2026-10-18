//! `Wall::check_ever_seen@0063ce70` under the emulator against the port (item 3234; tools/sweep_gen.py, obj with windows).

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

/// `Wall::check_ever_seen@0063ce70`, run by `tools/emu/sweep_wall_check_ever_seen.py` under unicorn
/// (tools/emu): 384 random bytes of object, 1 argument(s), `p` windows in order: p0..p1 the place(s), in flag order; the global cell 0x00c061ec points at 2048 random bytes; the global cell 0x00c06188 points at 512 random bytes; 227168 random bytes at 0x00e3a390; callee 0x00643440 (2 args) stubbed; callee 0x006e1250 (3 args) stubbed; vtable slot +0x164 (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; the word at `c1+0x15c` points at 4096 random bytes less `0` (placed); the word at `m+0x18` points at 576 random bytes less `0` (placed); slot +0x50 laid as 0x00472360 or its stub, a flag after the answers; slot +0x4c laid as 0x00472350 or its stub, a flag after the answers; stub 3 answered from [0, 1, 2] (90%); stub 4 answered from [0, 1, 4] (90%); stub 3 answers 17 values in call order, printed after the answers; stub 4 answers 16 values in call order, printed after the answers; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes a value in 0..40 through its argument 0, printed after the answers (`o` in a place); stub 0 writes a value in 0..40 through its argument 1, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_check_ever_seen.py") else {
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
            {
                // Stub answers: `a[1..6]` the five stubs' first answers (unused); `a[6]`/`a[7]` the
                // corner `tile_corner` writes (x, y); `a[8..25]` slot `+0x50` (`is_started`) in call
                // order, `a[25..41]` slot `+0x4c` (`is_active`); `a[41]`/`a[42]` say the slot held the
                // real `is_started` (`flags & 2`) / `is_active` (`flags & 4`) and not the stub.
                // Windows: c0 the game (`everyone_mask` at +0x6dc), c1 the world (stride +0xc, plane
                // pointer +0x15c), c2 the leaders (stride 0x6eec; flags +0, `met` words +0x94, ally
                // mask +0x6929), p0 the seen plane, p1 the wall's type (`x_size` +0x234, `y_size` +0x238).
                const K: i32 = 0x63637;
                const LS: usize = 0x6eec;
                let (mut ns, mut na) = (0usize, 0usize);
                let mut started = |m: &[u8]| -> bool {
                    if a[41] != 0 {
                        m[8] & 2 != 0
                    } else {
                        ns += 1;
                        a[8 + (ns - 1).min(16)] != 0
                    }
                };
                let mut active = |m: &[u8]| -> bool {
                    if a[42] != 0 {
                        m[8] & 4 != 0
                    } else {
                        na += 1;
                        a[25 + (na - 1).min(15)] != 0
                    }
                };
                let who = m[9] as usize;
                let mask_of = |c: &[Vec<u8>], o: usize| u32::from(c[2][o * LS + 0x6929]);
                let old = m[0x62];
                let want = if started(m) {
                    rd(&c[0], 0x6dc) as u32
                } else {
                    mask_of(c, who)
                };
                if (u32::from(m[0x62]) & want) != want || (u32::from(m[0x63]) & want) != want {
                    let (x, y) = (a[6], a[7]);
                    let stride = rd(&c[1], 0xc);
                    let (xs, ys) = (rd(&p[1], 0x234), rd(&p[1], 0x238));
                    for i in 0..xs {
                        let ix = (i + x) >> 1;
                        for j in 0..ys {
                            let iy = (j + y) >> 1;
                            let cell = p[0][(stride * iy + ix) as usize];
                            if !started(m) {
                                m[0x62] |= cell & (mask_of(c, who) as u8);
                            } else {
                                m[0x62] |= cell;
                                if active(m) {
                                    m[0x63] |= cell;
                                }
                            }
                        }
                    }
                }
                let (mut meets, mut lit) = (Vec::new(), false);
                if old != m[0x62] || a[0] != 0 {
                    for o in 0..8usize {
                        let mk = mask_of(c, o) as u8;
                        if o != who && c[2][o * LS] & 1 != 0 && m[0x62] & mk != 0 && old & mk == 0 {
                            lit = true;
                            if rd(&c[2], who * LS + 0x94 + o * 4) & 1 == 0 {
                                meets.push(format!(
                                    "c2+{:#x},{},{},{}",
                                    who * LS,
                                    o,
                                    rd(m, 0x10) ^ K,
                                    rd(m, 0x14) ^ K
                                ));
                            }
                        }
                    }
                }
                log.push(if meets.is_empty() {
                    "-".into()
                } else {
                    meets.join("/")
                });
                log.push(if lit { "m+0x0".into() } else { "-".into() });
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "check_ever_seen — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "check_ever_seen — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "check_ever_seen — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "check_ever_seen — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Wall::check_ever_seen@0063ce70: {n} rows agree");
}
