//! `Roads::scan_and_kill_stray_roads@008956a0` under the emulator against the port (item 3239; tools/sweep_gen.py, obj with windows).

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

/// `Roads::scan_and_kill_stray_roads@008956a0`, run by `tools/emu/sweep_roads_scan_and_kill_stray_roads.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), `p` windows in order: p0..p6 the place(s), in flag order; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c06238 points at 1792 random bytes; the global cell 0x00c061d0 points at 16 random bytes; the global cell 0x00c06218 points at 27316 random bytes; 36 random bytes at 0x00adc400 (each word in -1..1); 36 random bytes at 0x00adcaf0 (each word in -1..1); 36 random bytes at 0x00af2570; callee 0x0088e100 (4 args) stubbed; callee 0x0088e050 (2 args) stubbed; the word at `c3+0x6ab0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `p0+4` points at 128 random bytes less `0` (placed); the word at `p0+8` points at 128 random bytes less `0` (placed); the word at `p0+12` points at 128 random bytes less `0` (placed); the word at `c0+0x138` points at 128 random bytes less `0` (placed); the word at `c0+0x13c` points at 256 random bytes less `0` (placed); the word at `p1+0x5c` laid as the address of `p6` half the rows; the word at `p2+0x5c` laid as the address of `p6` half the rows; the word at `p3+0x5c` laid as the address of `p6` half the rows; the word at `p4+0x5c` laid as the address of `p6` half the rows; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 0 prints the 4 bytes its argument 2 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 0 prints the 144 bytes at the cell 0x20000794 when it is called, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 300 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_roads_scan_and_kill_stray_roads.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(7);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The Roads/World/WorldC/Terrain singletons are the cells c0..c3; the 36-byte tables the
            // sweep reads at 0xadc400 / 0xadcaf0 (c4, c5) are the tile offsets — the decompile's
            // `move_x` is the table at 0xadcaf0 (tile x, `c5`) and `move_y` the one at 0xadc400 (`c4`),
            // swapped in the decompile — and c6 the compass flags. p0 is the terrain's pointer table
            // (record `j` is `p[1 + j]`), p5 the tile words, p6 the candidates the records alias.
            let sh = |v: &[u8], o: usize| i32::from(u16::from_le_bytes([v[o], v[o + 1]]));
            let cnt = rd(&c[0], 8) / 500;
            let (mut l0, mut l1): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
            let (mut d00, mut d01, mut d10, mut d11, mut d4): (
                Vec<String>,
                Vec<String>,
                Vec<String>,
                Vec<String>,
                Vec<String>,
            ) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
            let hx = |v: i32| {
                v.to_le_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            };
            for _ in 0..cnt.max(0) {
                let x = rd(&c[1], 0x5fc) + 1;
                wr(&mut c[1], 0x5fc, x);
                if x >= rd(&c[0], 0) {
                    wr(&mut c[1], 0x5fc, 0);
                    let y = rd(&c[1], 0x600) + 1;
                    wr(&mut c[1], 0x600, y);
                    if y >= rd(&c[0], 4) {
                        wr(&mut c[1], 0x600, 0);
                    }
                }
                let idx = usize::try_from(rd(&c[1], 0x600) * rd(&c[2], 0) + rd(&c[1], 0x5fc))
                    .expect("cell");
                let rec = 1 + idx;
                let cand = rd(&p[rec], 0x5c) != 0;
                for i in 0..16usize {
                    let tx = (i as i32 & 3) + rd(&c[1], 0x5fc) * 4;
                    let ty = (i as i32 >> 2) + rd(&c[1], 0x600) * 4;
                    for k in 0..9usize {
                        let (px, py) = (rd(&c[5], 4 * k) + tx, rd(&c[4], 4 * k) + ty);
                        let (w18, w1c) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
                        if px < 0 || py < 0 || px >= w18 || py >= w1c {
                            wr(&mut c[1], 0x128 + 4 * k, 0);
                            wr(&mut c[1], 0x628 + 4 * k, 0);
                            wr(&mut c[1], 0x670 + 4 * k, 0);
                            wr(&mut c[1], 0x64c + 4 * k, 0);
                            continue;
                        }
                        let t = sh(&p[5], usize::try_from(w18 * py + px).expect("tile") * 2);
                        let road = (t & 0x30) == 0x10;
                        wr(&mut c[1], 0x604 + 4 * k, i32::from(road));
                        wr(&mut c[1], 0x628 + 4 * k, i32::from((t & 3) == 3));
                        wr(
                            &mut c[1],
                            0x670 + 4 * k,
                            i32::from(t & 0x800 != 0 || (t & 0x30) == 0x20),
                        );
                        let sup = road
                            && cand
                            && p[6][16 * i + 8] != 10
                            && (rd(&c[6], 4 * k) & rd(&p[6], 16 * i + 4)) != 0;
                        wr(&mut c[1], 0x64c + 4 * k, i32::from(sup));
                    }
                    l0.push(format!("c1+0x0,p{rec}+0x0,sp-0xc,sp-0x1c,{i}"));
                    d00.push(hx(tx));
                    d01.push(hx(ty));
                    // The roads record's 0x604..0x694 as the callee is entered (the arrays this sweep fills).
                    d4.push(
                        c[1][0x604..0x694]
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>(),
                    );
                    if rd(&c[1], 0x604) != 0 {
                        l1.push("c1+0x0,sp-0xc,sp-0x1c".to_string());
                        d10.push(hx(tx));
                        d11.push(hx(ty));
                    }
                }
            }
            for v in [l0, l1, d00, d01, d10, d11, d4] {
                log.push(if v.is_empty() {
                    "-".to_string()
                } else {
                    v.join("/")
                });
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "scan_and_kill_stray_roads — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "scan_and_kill_stray_roads — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "scan_and_kill_stray_roads — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[8 + i],
                "scan_and_kill_stray_roads — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("Roads::scan_and_kill_stray_roads@008956a0: {n} rows agree");
}
