//! `Unit::update_z@00606590` under the emulator against the port (item 3044; tools/sweep_gen.py, obj with windows).

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

/// `Unit::update_z@00606590`, run by `tools/emu/sweep_unit_update_z.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p14 the place(s), in flag order; 4 random bytes at 0x00c06218; callee 0x008544a0 (3 args) stubbed; callee 0x00866560 (4 args) stubbed; the word at `0xcae5fc` points at 1024 random bytes less `0` (placed); the word at `m+0xf4` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 256 random bytes less `0` (placed); the word at `p1+4` points at 256 random bytes less `0` (placed); the word at `p1+8` points at 256 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 256 random bytes less `0` (placed); the word at `p5+4` points at 256 random bytes less `0` (placed); the word at `p6+0` points at 256 random bytes less `0` (placed); the word at `p6+4` points at 256 random bytes less `0` (placed); the word at `p7+0x18` points at 544 random bytes less `0` (placed); the word at `p8+0x18` points at 544 random bytes less `0` (placed); the word at `p9+0x18` points at 544 random bytes less `0` (placed); the word at `p10+0x18` points at 544 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 writes a value in 0..1048575 through its argument 0, printed after the answers (`o` in a place); stub 1 writes a value in 0..1048575 through its argument 0, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_update_z.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `m+0x10`/`+0x14` the masked position, its tile (>> 6) indexing the div-3 table `p[0]`; stub 0
            // (`find_tcoord_z`, answer `a[2]`) gives `+0xc`. Then each child (`m+0xe8` of them, list `p[1]`,
            // child `j` in `p[2+j]`): its type record is `p[11 + 2 * player + index]`, whose `+0x218`
            // of 1 zeroes the child's `+0x14`, else stub 1 (`find_data_z`, answer `a[3]`) gives it.
            const K: i32 = 0x63637;
            let terrain = rd(&c[0], 0);
            let x = rd(m, 0x10) ^ K;
            let y = rd(m, 0x14) ^ K;
            let tx = rd(&p[0], 4 * (x >> 6) as usize);
            let ty = rd(&p[0], 4 * (y >> 6) as usize);
            log.push(format!("s0:{terrain},sp-0xc,{tx},{ty}"));
            wr(m, 0xc, a[2] ^ K);
            for j in 0..rd(m, 0xe8).max(0) as usize {
                let (player, index) = (p[2 + j][0xa1] as i8 as usize, rd(&p[2 + j], 0x8c) as i16 as usize);
                if rd(&p[11 + 2 * player + index], 0x218) == 1 {
                    wr(&mut p[2 + j], 0x14, 0);
                } else {
                    let (cx, cy) = (rd(&p[2 + j], 0xc), rd(&p[2 + j], 0x10));
                    log.push(format!("s1:{terrain},sp-0xc,{cx},{cy},0"));
                    wr(&mut p[2 + j], 0x14, a[3]);
                }
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "update_z — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_z — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "update_z — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "update_z — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Unit::update_z@00606590: {n} rows agree");
}
