//! `Build::update_max_gatherers@00623310` under the emulator against the port (item 2336; tools/sweep_gen.py, obj with windows).

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

/// `Build::update_max_gatherers@00623310`, run by `tools/emu/sweep_build_update_max_gatherers.py` under unicorn
/// (tools/emu): 144 random bytes of object, 0 argument(s), callee 0x00643440 (2 args) stubbed; callee 0x0063c430 (4 args) stubbed; the word at `m+0x18` points at 16 random bytes less `0` (placed); stub 1 answered from [0, 1, 2, 3, 255, 256, 4660] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes a value in -5..300 through its argument 0, printed after the answers (`o` in a place); stub 0 writes a value in -5..300 through its argument 1, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_update_max_gatherers.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `tile_corner` (stub 0) writes the corner's two tile coordinates into two locals
                // (`o`); the type's (+0x18) `max_gatherers` (stub 1) is asked (the short at +0xa, the
                // byte at +9, the two coordinates), and its low byte is kept at +0x80.
                log.push("m+0x0,sp-0x10,sp-0xc".into());
                let short = i16::from_le_bytes([m[0xa], m[0xb]]);
                log.push(format!("p0+0x0,{},{},{},{}", short, m[9], a[2], a[3]));
                m[0x80] = a[1] as u8;
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "update_max_gatherers — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "update_max_gatherers — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "update_max_gatherers — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "update_max_gatherers — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Build::update_max_gatherers@00623310: {n} rows agree");
}
