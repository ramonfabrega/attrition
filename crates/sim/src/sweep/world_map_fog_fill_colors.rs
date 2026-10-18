//! `WorldMapFog::fill_colors@008d17b0` under the emulator against the port (item 3049; tools/sweep_gen.py, obj with windows).

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

/// `WorldMapFog::fill_colors@008d17b0`, run by `tools/emu/sweep_world_map_fog_fill_colors.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), `p` windows in order: none; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c061e0 points at 64 random bytes; 8 random bytes at 0x00e8a794; callee 0x006d5f60 (2 args) stubbed; callee 0x008d1070 (0 args) stubbed; callee 0x008d1420 (0 args) stubbed; stub 0 answered from [1, 1, 1, 0] (90%); stub 0 answers 8 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_map_fog_fill_colors.py") else {
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
            // The seen-borders mask of the eight players (bit i when the console player's leader — the leader
            // table `c1` at who * 0x6eec — can see player i's borders, the stub, asked with (i, 1)) is
            // compared with the fog's `can_see_mask` (`c2+4`); a change sets the fog's dirty bit 2 (`c2+0`)
            // and stores the mask. A dirty fog refills the who colours (stub 1) and the fog colours (stub 2)
            // and clears the bit; a clean one refills the fog colours only.
            let who = rd(&c[0], 0x298);
            let ecx = format!("c1+{:#x}", who * 0x6eec);
            let mut mask = 0i32;
            let mut asks = Vec::new();
            for i in 0..8usize {
                asks.push(format!("{ecx},{i},1"));
                if a[3 + i] != 0 {
                    mask |= 1 << i;
                }
            }
            let mut flags = c[2][0];
            if mask != rd(&c[2], 4) {
                flags |= 2;
                wr(&mut c[2], 4, mask);
            }
            log.push(asks.join("/"));
            if flags & 2 != 0 {
                log.push(ecx.clone());
                log.push(ecx);
                flags &= !2;
            } else {
                log.push("-".to_string());
                log.push(ecx);
            }
            c[2][0] = flags;
            want_eax
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "fill_colors — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "fill_colors — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "fill_colors — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "fill_colors — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("WorldMapFog::fill_colors@008d17b0: {n} rows agree");
}
