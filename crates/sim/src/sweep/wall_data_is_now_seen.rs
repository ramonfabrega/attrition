//! `WallData::is_now_seen@00642ca0` under the emulator against the port (item 2927; tools/sweep_gen.py, obj with windows).

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

/// `WallData::is_now_seen@00642ca0`, run by `tools/emu/sweep_wall_data_is_now_seen.py` under unicorn
/// (tools/emu): 64 random bytes of object, 1 argument(s), the global cell 0x00c061d0 points at 8 random bytes; the global cell 0x00c061e8 points at 2112 random bytes; the global cell 0x00cae5fc points at 256 random bytes; vtable slot +0x50 (0 args) stubbed; callee 0x006b55c0 (3 args) stubbed; slot +0x50 laid as 0x00472360 or its stub, a flag after the answers; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); 12 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x1, +0x6929:0x1 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_data_is_now_seen.py") else {
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
            'p: {
                // is_now_seen: the owner sees its own wall; the others if it is started (slot 0x50) or the viewer's leader has the
                // owner's bit in the byte at +0x6929, unless the map is revealed (gamec +0x30 == 3) or the owner's fog is off; else
                // the world is asked (is_seen) about the tile of the wall, whose answer is returned whole.
                let _ = &c[2];
                log.extend(["-".to_string(), "-".to_string(), "-".to_string()]);
                let player = a[0];
                let owner = m[9] as i32;
                if player == owner {
                    break 'p 1;
                }
                let started = if a[3] != 0 { m[8] & 2 != 0 } else { a[1] != 0 };
                let lead = |row: i32, k: usize| p[2 * row as usize + k][0];
                if !started
                    && (lead(player, 1) & (1u32.checked_shl(owner as u32 & 31).unwrap_or(0) as u8))
                        == 0
                {
                    break 'p 0;
                }
                if c[1][0x30] == 3
                    || (m[8] & 0x20 != 0 && c[1][0x820] & 4 == 0 && lead(owner, 0) & 4 != 0)
                {
                    break 'p 1;
                }
                let tab = |w: i32| {
                    let i = ((w ^ 0x63637) >> 7) as usize;
                    rd(&c[2], 4 * i)
                };
                let x = tab(rd(m, 0x10));
                let y = tab(rd(m, 0x14));
                log.clear();
                log.push(format!("c0+0x0,sp-0x10,sp-0x4,{player}"));
                log.push(x.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect());
                log.push(y.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect());
                a[2] as i64
            }
        };
        assert_eq!(got, want_eax, "is_now_seen — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "is_now_seen — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_now_seen — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_now_seen — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "is_now_seen — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WallData::is_now_seen@00642ca0: {n} rows agree");
}
