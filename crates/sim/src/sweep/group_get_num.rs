//! `Group::get_num@00714700` under the emulator against the port (item 3024; tools/sweep_gen.py, obj with windows).

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

/// `Group::get_num@00714700`, run by `tools/emu/sweep_group_get_num.py` under unicorn
/// (tools/emu): 2516 random bytes of object, 0 argument(s), `p` windows in order: p0..p9 the place(s), in flag order; the global cell 0x00c0618c points at 64 random bytes; callee 0x00711540 (0 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_get_num.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The group: `+0xc` its count, `+4` a sign that sends a short group to `normalize`
            // (the stubbed direct callee), the owner byte `+0x4a`, then parallel arrays: shorts
            // at `+0x8cc`, bytes at `+0x84c`, dwords at `+0x4c`, `+0x24c`, `+0x44c`, `+0x64c`.
            // The objects table p[2 + 4 * owner + short]'s byte `+8` bit 0 says a member lives;
            // a dead one is shifted out of the four arrays and the count drops.
            log.push("-".into());
            let count = rd(m, 0xc);
            if count <= 0 {
                for o in [0xc, 0x1c, 0x18] {
                    wr(m, o, 0);
                }
            } else if count < 4 && rd(m, 4) >= 0 {
                log[0] = "m+0x0".into();
            } else {
                let owner = m[0x4a] as usize;
                for i in (0..count as usize).rev() {
                    let at = 0x8cc + 2 * i;
                    let k = i16::from_le_bytes([m[at], m[at + 1]]) as usize;
                    if p[2 + 4 * owner + k][8] & 1 != 0 {
                        continue;
                    }
                    let cur = rd(m, 0xc) as usize;
                    for j in i..cur.saturating_sub(1) {
                        m.copy_within(0x8cc + 2 * (j + 1)..0x8cc + 2 * (j + 1) + 2, 0x8cc + 2 * j);
                        m[0x84c + j] = m[0x84c + j + 1];
                        for base in [0x4c, 0x24c, 0x44c, 0x64c] {
                            let v = rd(m, base + 4 * (j + 1));
                            wr(m, base + 4 * j, v);
                        }
                    }
                    wr(m, 0xc, cur as i32 - 1);
                }
            }
            i64::from(rd(m, 0xc))
        };
        assert_eq!(got, want_eax, "get_num — eax, row `{line}`");
        assert_eq!(log, want_log, "get_num — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_num — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_num — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "get_num — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Group::get_num@00714700: {n} rows agree");
}
