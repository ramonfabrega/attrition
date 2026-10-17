//! `Wall::swap_team@00640c00` under the emulator against the port (item 3030; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn return_eax(v: i32) -> i64 {
    i64::from(v)
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `Wall::swap_team@00640c00`, run by `tools/emu/sweep_wall_swap_team.py` under unicorn
/// (tools/emu): 112 random bytes of object, 1 argument(s), `p` windows in order: p0..p11 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c061ec points at 32 random bytes; 4 random bytes at 0x00e7fe4c (each byte in 0..2); vtable slot +0x4c (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x15c (1 args) stubbed; vtable slot +0x160 (0 args) stubbed; callee 0x006e3140 (1 args) stubbed; callee 0x0065d190 (6 args) stubbed; callee 0x00627fe0 (0 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 32 random bytes less `0` (placed); the word at `p5+4` points at 32 random bytes less `0` (placed); the word at `p5+8` points at 32 random bytes less `0` (placed); the word at `p5+12` points at 32 random bytes less `0` (placed); the word at `m+0x18` points at 64 random bytes less `0` (placed); stub 3's answer points at 128 random bytes less `0` (placed); slot +0x4c laid as 0x00472350 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 8 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stack 256 bytes under the call is zeroed before each row; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_swap_team.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            let k = 0x63637;
            let who = a[0];
            let (mut t0, mut t1, mut t2, mut t4, mut t9) = ("-".to_string(), "-".to_string(), "-".to_string(), "-".to_string(), "-".to_string());
            let t7 = format!("{},{}", 0xe3a390 + who * 0x6eec, rd(&p[10], 4));
            let t8 = format!("c0+0x0,{who},{},{},{},1,-1", a[8], rd(m, 0x10) ^ k, rd(m, 0x14) ^ k);
            let ebx = a[9];
            let mut t5 = "-".to_string();
            let mut t6 = "-".to_string();
            let tail = |t7: String, t8: String, t0, t1, t2, t4, t5, t6, t9| -> Vec<String> {
                vec![t0, t1, t2, t4, t5, t6, t7, t8, t9]
            };
            if ebx < 0 {
                log.extend(tail(t7, t8, t0, t1, t2, t4, t5, t6, t9));
                return_eax(ebx)
            } else {
                let (ml, hd) = (m[0x60], m[0x61]);
                p[11][0x60] = ml;
                p[11][0x61] = hd;
                p[11][0x62] = m[0x62];
                p[11][0x63] = m[0x63];
                let gv = c[1][0xc..0x10].to_vec();
                let xv = c[2][0..4].to_vec();
                let gt = gv[0] > xv[0]
                    || (gv[0] == xv[0]
                        && (gv[1] > xv[1]
                            || (gv[1] == xv[1] && (gv[2] > xv[2] || (gv[2] == xv[2] && gv[3] > xv[3])))));
                if gt {
                    let active = if a[11] == 1 {
                        i32::from(m[8] & 4)
                    } else {
                        t0 = "m+0x0".to_string();
                        a[1]
                    };
                    if active != 0 {
                        let iv = if a[12] == 1 {
                            t2 = "p10+0x0,435,0".to_string();
                            a[3]
                        } else {
                            t1 = "m+0x0,435,0".to_string();
                            a[2]
                        };
                        if iv != 0 {
                            t4 = format!("p{}+0x0", 1 + 5 * who + ebx);
                            t9 = a[5].to_string();
                        }
                    }
                }
                let old8 = p[11][8];
                p[11][8] = (old8 & 0x40) | (m[8] & 0xbf);
                for o in [0x48usize, 0x4c, 0x50, 0x54, 0x24] {
                    let v = rd(m, o);
                    wr(&mut p[11], o, v);
                }
                p[11][0x3a] = m[0x3a];
                p[11][0x40] = m[0x40];
                t5 = "p11+0x0,0".to_string();
                t6 = "p11+0x0".to_string();
                log.extend(tail(t7, t8, t0, t1, t2, t4, t5, t6, t9));
                return_eax(ebx)
            }
        };
        assert_eq!(got, want_eax, "swap_team — eax, row `{line}`");
        assert_eq!(log, want_log, "swap_team — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "swap_team — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "swap_team — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "swap_team — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("Wall::swap_team@00640c00: {n} rows agree");
}
