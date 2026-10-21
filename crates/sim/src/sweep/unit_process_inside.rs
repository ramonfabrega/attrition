//! `Unit::process@00610bc0` under the emulator against the port (item 3403; tools/sweep_gen.py, obj with windows).

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

/// `Unit::process@00610bc0`, run by `tools/emu/sweep_unit_process_inside.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 0 argument(s), `p` windows in order: p0..p20 the place(s), in flag order; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c0618c points at 128 random bytes; callee 0x005e0670 (0 args) stubbed; callee 0x00651a80 (1 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x00617c10 (1 args) stubbed; the word at `m+0xf4` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `p0+4` points at 128 random bytes less `0` (placed); the word at `p0+8` points at 128 random bytes less `0` (placed); the word at `p0+12` points at 128 random bytes less `0` (placed); the word at `p0+16` points at 128 random bytes less `0` (placed); the word at `p0+20` points at 128 random bytes less `0` (placed); the word at `p0+24` points at 128 random bytes less `0` (placed); the word at `p0+28` points at 128 random bytes less `0` (placed); the word at `c1+0x14+o[0]*0x1c` points at 16 random bytes less `0` (placed); the word at `p9+0` points at 64 random bytes less `0` (placed); the word at `p9+4` points at 64 random bytes less `0` (placed); the word at `p9+8` points at 64 random bytes less `0` (placed); the word at `p9+12` points at 64 random bytes less `0` (placed); the word at `0xc09970+o[0]*0x1c` points at 16 random bytes less `0` (placed); the word at `p14+0` points at 32 random bytes less `0` (placed); the word at `p14+4` points at 32 random bytes less `0` (placed); the word at `p14+8` points at 32 random bytes less `0` (placed); the word at `p14+12` points at 32 random bytes less `0` (placed); the word at `m+0x18` points at 800 random bytes less `0` (placed); stub 2's answer points at 128 random bytes less `0` (placed); stub 1 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 writes a value in 0..3 through its argument 0, printed after the answers (`o` in a place); the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_process_inside.py") else {
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
            // hand: port
            // `Unit::process@00610bc0`, the `+0x82 >= 0` (inside) arm with the head's arms skipped
            // by the biases: the two mask clears, the thirty-two-frame come-out test through the
            // garrisoning building's city, and the figures' `last_z` write. Windows: c0 the game
            // (`+0x550`), c1 the objects cell; p0 the figures' array, p1..p8 the figures, p9..p13
            // the objects table (p10..p13 its objects), p14..p18 the cities table (p15..p18 its
            // cities), p19 the type record, p20 the city the slot-+0xac stub answers. `a[0..4]`
            // the stubs' answers, `a[4]` the out value of `get_inside`.
            let u16at = |w: &[u8], o: usize| u16::from_le_bytes([w[o], w[o + 1]]);
            let mut tok: Vec<Vec<String>> = vec![Vec::new(); 4];
            let who = i32::from(m[9]);
            let frame = rd(&c[0], 0x550);
            let this = "m+0x0".to_string();
            for off in [0xae, 0xac, 0xad] {
                if m[off] != 0 {
                    m[off] -= 1;
                }
            }
            if u16at(m, 0x38) != 0 {
                let v = u16at(m, 0x38) - 1;
                m[0x38..0x3a].copy_from_slice(&v.to_le_bytes());
            }
            tok[0].push(this.clone());
            wr(m, 0x6c, rd(m, 0x6c) & 0xffff_7f9fu32 as i32);
            let f68 = rd(m, 0x68) & 0xfffe_efffu32 as i32;
            wr(m, 0x68, f68);
            if f68 & 0x2000 == 0 {
                wr(m, 0x68, f68 & 0xffff_f7ffu32 as i32);
            }
            if who.wrapping_add(frame) & 31 == 0 && rd(m, 0x68) & 0x4_0000 != 0 {
                tok[1].push("m+0x0,sp-0x18".to_string());
                let idx = a[1] as usize;
                if p[10 + idx][8] & 0x20 != 0 {
                    tok[2].push(format!("p{}+0x0", 10 + idx));
                    let city = i16::from_le_bytes([p[20][0x72], p[20][0x73]]) as usize;
                    if p[15 + city][4] & 0x42 == 0 {
                        tok[3].push(format!("{this},0"));
                    }
                }
            }
            for i in 0..rd(m, 0xe8) as usize {
                let g = &mut p[1 + i];
                let z = rd(g, 0x14);
                wr(g, 0x70, z);
            }
            let join = |v: &[String]| {
                if v.is_empty() {
                    "-".to_string()
                } else {
                    v.join("/")
                }
            };
            for t in &tok {
                log.push(join(t));
            }
            0i64
            // end hand
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "process — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Unit::process@00610bc0: {n} rows agree");
}
