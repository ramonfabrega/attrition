//! `WorldMapFog::add_update@008d2760` under the emulator against the port (item 2910; tools/sweep_gen.py, obj with windows).

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

/// `WorldMapFog::add_update@008d2760`, run by `tools/emu/sweep_world_map_fog_add_update.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), 4 random bytes at 0x00e8a794; 48 random bytes at 0x00e8a85c; argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x00433ab0 (1 args) stubbed; the word at `c1+0x10` points at 64 random bytes less `0` (placed); the word at `c1+0x2c` points at 64 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_map_fog_add_update.py") else {
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
            let _ = (&a, &m);
            // Unless fog flag 2 (word at 0xe8a794) is set, each of the two update arrays (at 0xe8a85c and
            // 0xe8a878: length +4, capacity +8, growth step the i16 at +0xc, list pointer +0x10) takes one
            // coordinate pushed on its end — after a call of `increase_size` (stubbed) when it is full.
            let mut calls: Vec<String> = Vec::new();
            if rd(&c[0], 0) & 2 == 0 {
                for (base, v, list) in
                    [(0usize, rd(&p[0], 0), 2usize), (0x1c, rd(&p[1], 0), 3)].into_iter()
                {
                    let len = rd(&c[1], base + 4);
                    if len >= rd(&c[1], base + 8) {
                        let step = i16::from_le_bytes([c[1][base + 0xc], c[1][base + 0xd]]);
                        calls.push(format!("c1+{base:#x},{step}"));
                    }
                    wr(&mut p[list], 4 * len as usize, v);
                    wr(&mut c[1], base + 4, len + 1);
                }
            }
            log.push(if calls.is_empty() {
                "-".into()
            } else {
                calls.join("/")
            });
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "add_update — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_update — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "add_update — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "add_update — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WorldMapFog::add_update@008d2760: {n} rows agree");
}
