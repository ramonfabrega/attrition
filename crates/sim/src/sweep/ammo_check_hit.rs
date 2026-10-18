//! `Ammo::check_hit@00678d90` under the emulator against the port (item 3200; tools/sweep_gen.py, obj with windows).

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

/// `Ammo::check_hit@00678d90`, run by `tools/emu/sweep_ammo_check_hit.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0..p7 the place(s), in flag order; the global cell 0x00c0618c points at 768 random bytes; the global cell 0x00c0aec0 points at 16 random bytes; the global cell 0x00c06188 points at 768 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; callee 0x0065ca80 (13 args) stubbed; callee 0x0065ab40 (7 args) stubbed; callee 0x006624f0 (4 args) stubbed; the word at `c0+0x14` points at 64 random bytes less `0` (placed); the word at `c1+0` points at 64 random bytes less `0` (placed); the word at `c1+4` points at 64 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p1+0x18` points at 784 random bytes less `0` (placed); the word at `p2+0x18` points at 784 random bytes less `0` (placed); the word at `p3+0x18` points at 768 random bytes less `0` (placed); the word at `c2+0x138` points at 17408 random bytes less `0` (placed); stub 1 answered from [-1, -1, -1, 0, 3, 5] (90%); stub 0 answers -1..1 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_ammo_check_hit.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // m `this` (the ammo); c0 the objects record (`+0x1fc` the minimum, `+0x200` the last
            // search's player), c1 the units cell (two units), c2 the world, c3 the div_3_table; p3 the
            // shooter's object (the sound's `ecx`), p6 its type, p4/p5 the two found units' types, p7
            // the tile flags. a[0] the domain, a[1] the `find_unit` answer, a[2] the
            // `find_building_at` answer. Recorded: the miss sound, `p3+0x0,<sound>,<x>,<y>,<tile x>`.
            let rdw = |w: &[u8], o: usize| {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            };
            let wrw =
                |w: &mut [u8], o: usize, v: i32| w[o..o + 4].copy_from_slice(&v.to_le_bytes());
            let mut ret = 1;
            let mut calls: Vec<String> = Vec::new();
            let found = a[1];
            wrw(m, 0x4c, found);
            wrw(m, 0x48, rdw(&c[0], 0x200));
            if found >= 0 {
                let ty = if found == 0 { &p[4] } else { &p[5] };
                if rdw(ty, 0x300) < rdw(&c[0], 0x1fc) {
                    wrw(m, 0x4c, -1);
                    wrw(m, 0x48, -1);
                }
            }
            if rdw(m, 0x4c) < 0 {
                let (x, y) = (rdw(m, 0x18), rdw(m, 0x1c));
                if x >= 0
                    && y >= 0
                    && x < rdw(&c[2], 0x18).wrapping_mul(0xc0)
                    && y < rdw(&c[2], 0x1c).wrapping_mul(0xc0)
                {
                    let b = a[2];
                    wrw(m, 0x4c, b);
                    wrw(m, 0x48, rdw(&c[0], 0x200));
                    if b >= 0 {
                        ret = 1;
                    } else {
                        let tx = rdw(&c[3], ((x >> 6) as usize) * 4);
                        let ty = rdw(&c[3], ((y >> 6) as usize) * 4);
                        let at = (rdw(&c[2], 0x18).wrapping_mul(ty).wrapping_add(tx)) as usize * 2;
                        let sound = if p[7][at] & 0x30 == 0x20 { 6 } else { 7 };
                        calls.push(format!(
                            "p3+0x0,{sound},{},{},{tx}",
                            tx * 0xc0 + 0x60,
                            ty * 0xc0 + 0x60
                        ));
                        wrw(m, 0x4c, -1);
                        wrw(m, 0x48, -1);
                        ret = 0;
                    }
                } else {
                    wrw(m, 0x48, -1);
                    ret = 0;
                }
            }
            log.push(if calls.is_empty() {
                "-".to_string()
            } else {
                calls.join("/")
            });
            ret
        };
        assert_eq!(got, want_eax, "check_hit — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "check_hit — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "check_hit — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "check_hit — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "check_hit — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("Ammo::check_hit@00678d90: {n} rows agree");
}
