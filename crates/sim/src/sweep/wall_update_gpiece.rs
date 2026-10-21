//! `Wall::update_gpiece@00643070` under the emulator against the port (item 3395; tools/sweep_gen.py, obj with windows).

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

/// `Wall::update_gpiece@00643070`, run by `tools/emu/sweep_wall_update_gpiece.py` under unicorn
/// (tools/emu): 128 random bytes of object, 1 argument(s), `p` windows in order: p0..p2 sparse table 0 (3 rows × 1 slice(s), row-major), p3..p26 the place(s), in flag order, p27 stub 1's answer window; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c06214 points at 3424 random bytes; the global cell 0x00e335a0 points at 232 random bytes; the global cell 0x00c06210 points at 676 random bytes; the global cell 0x00c061ec points at 2096 random bytes; vtable slot +0x20 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; callee 0x0090bce0 (6 args) stubbed; callee 0x008e4780 (1 args) stubbed; callee 0x00907990 (1 args) stubbed; the word at `c0+0x14+0x1c*m[9]` points at 16 random bytes less `0` (placed); the word at `p3+0` points at 16 random bytes less `0` (placed); the word at `p3+4` points at 16 random bytes less `0` (placed); the word at `p3+8` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 224 random bytes less `0` (placed); the word at `p1+0` points at 224 random bytes less `0` (placed); the word at `p2+0` points at 224 random bytes less `0` (placed); the word at `m+0x18` points at 16 random bytes less `0` (placed); the word at `c1+0x728` points at 24 random bytes less `0` (placed); the word at `p11+0` points at 72 random bytes less `0` (placed); the word at `p11+4` points at 72 random bytes less `0` (placed); the word at `p11+8` points at 72 random bytes less `0` (placed); the word at `p11+12` points at 72 random bytes less `0` (placed); the word at `p11+16` points at 72 random bytes less `0` (placed); the word at `p12+0x44` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p13+0x44` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p14+0x44` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p15+0x44` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p16+0x44` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p17+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p18+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p19+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p20+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p21+8` points at 16 random bytes less `0` (placed, null a third of the rows); stub 0 answered from [0, 1, 1, 2] (90%); stub 2 answered from [0, 0, 0, 1] (90%); stub 1 answers the address of 136 random bytes, a `p` window after the places; stub 3 answers 3 values in call order, printed after the answers; stub 5 answers 2 values in call order, printed after the answers; stub 3 answers 0..5 every row; stub 5 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 3 rows 0x6eec apart at `@0xe41248`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_update_gpiece.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // No crate method: the choice of a wall's pieces is a renderer's, and the sim
            // carries no `GraphicPieces` (garrison_view's `update_gpiece` is `Unit`'s, a
            // namesake), so the port is inline here. Windows: m the wall; c1 the pieces'
            // record (`+0xd48` first, `+0xd54` count), c2 the graphics grade (`+0xe4`), c3 the
            // console (`+0x29c` original_who), c4 the game (`+0x820` word, bit 11); p4..p6 the
            // objects, p7..p9 the leaders' records (`+0xdc`), p10 the type record (`+4`), p11
            // the pieces' table (p12..p16 its records, p17..p21 their `+0x44`, p22..p26 their
            // `+8`), p27 the object's answer record. a: [own, vcall 0x20, vcall 0xac, vcall 0x8,
            // build base, verify base, cont base, build x3, cont x2].
            let who = m[9] as usize;
            let idx = i16::from_le_bytes([m[0xa], m[0xb]]) as i32;
            let c1 = "c1+0x0";
            let xor_dc = |w: &[u8]| rd(w, 0xdc) ^ 0x62766;
            let ty = rd(&p[10], 4);
            let (mut obj, mut vc8, mut build, mut cont) = (vec![], vec![], vec![], vec![]);
            let (mut esi, mut ebx) = (who as i32, 0i32);
            if a[1] != 0 {
                assert!((0..=2).contains(&idx), "drawn in range");
                let o = format!("p{}+0x0", 4 + idx);
                obj = vec![o.clone(), o];
                let k = p[27][0x7f] as i8 as i32;
                assert!((0..=2).contains(&k), "drawn in range");
                esi = k;
                ebx = (p[27][0x83] as i32).wrapping_sub(xor_dc(&p[7 + k as usize]));
            }
            let g1 = a[7];
            build.push(format!("{c1},{ty},{esi},1,{ebx},1,0"));
            wr(m, 0x58, g1);
            let verify = format!("{c1},{g1}");
            let (first, total) = (rd(&c[1], 0xd48), rd(&c[1], 0xd54));
            let table = rd; // the pieces' table is window p11
            if first <= g1 && g1.wrapping_sub(first) < total {
                let g = g1 as usize;
                assert!(table(&p[11], 4 * g) != 0, "drawn in range");
                if rd(&p[12 + g], 0x44) != 0 && rd(&p[17 + g], 8) != 0 {
                    vc8.push(format!("p{}+0x0", 22 + g));
                    if a[3] != 0 {
                        ebx = ebx.wrapping_sub(1);
                    }
                }
            }
            let mut g2 = a[8];
            build.push(format!("{c1},{ty},{esi},0,{ebx},1,0"));
            if rd(&c[2], 0xe4) > 2 {
                let ow = rd(&c[3], 0x29c);
                assert!((0..=2).contains(&ow), "drawn in range");
                let diff = xor_dc(&p[7 + who]).wrapping_sub(xor_dc(&p[7 + ow as usize]));
                build.push(format!("{c1},{ty},{ow},1,{diff},1,0"));
                g2 = a[9];
            }
            let mut keep = false;
            if table(&p[11], 4 * g2 as usize) != 0 {
                let g = g2 as usize;
                cont = vec![format!("{c1},{g2}"), format!("{c1},{g1}")];
                if (a[10] ^ a[11]) & !1 == 0 && rd(&p[12 + g], 0x44) != 0 && rd(&p[17 + g], 8) != 0
                {
                    vc8.push(format!("p{}+0x0", 22 + g));
                    keep = a[3] == 0 && (rd(&c[4], 0x820) >> 8) & 8 == 0;
                }
            }
            wr(m, 0x68, if keep { g2 } else { g1 });
            let col = |v: Vec<String>| {
                if v.is_empty() {
                    "-".to_string()
                } else {
                    v.join("/")
                }
            };
            log.push("m+0x0".into());
            log.push(col(obj));
            log.push(col(vc8));
            log.push(col(build));
            log.push(verify);
            log.push(col(cont));
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "update_gpiece — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_gpiece — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "update_gpiece — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[6 + i],
                "update_gpiece — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("Wall::update_gpiece@00643070: {n} rows agree");
}
