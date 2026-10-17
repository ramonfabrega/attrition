//! `ItemData::is_seen@00677850` under the emulator against the port (item 2877; tools/sweep_gen.py, obj with windows).

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

/// `ItemData::is_seen@00677850`, run by `tools/emu/sweep_item_data_is_seen.py` under unicorn
/// (tools/emu): 48 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 316 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; callee 0x006e1370 (1 args) stubbed; callee 0x006b53f0 (3 args) stubbed; callee 0x006b55c0 (3 args) stubbed; the word at `c0+0x134` points at 1536 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); 4 rows 0x6eec apart at `0xc061e0`, slices +0x6929:0x1 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_item_data_is_seen.py") else {
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
            {
                let player = a[0];
                let ever = m[0x20];
                let mask = p[player as usize][0];
                let mut l: [String; 6] = std::array::from_fn(|_| "-".to_string());
                let d = |v: i32, sh: u32| rd(&c[1], (((v ^ 0x63637) >> sh) as usize) * 4);
                let h = |v: i32| {
                    v.to_le_bytes()
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                };
                let (x, y) = (rd(m, 0x10), rd(m, 0x14));
                let mut seen_via = 2usize;
                let eax: i64 = if ever & mask != 0 {
                    1
                } else {
                    if player >= 0 && a[2] != 0 {
                        let idx = d(y, 8).wrapping_mul(rd(&c[0], 0)).wrapping_add(d(x, 8)) as usize;
                        if i32::from(p[4][idx.wrapping_mul(0x1c) + 0xf] as i8) == player {
                            seen_via = 1;
                        }
                    }
                    let k = seen_via;
                    l[k - 1] = format!("c0+0x0,sp-0x14,sp-0x8,{player}");
                    l[2 * k] = h(d(x, 7));
                    l[2 * k + 1] = h(d(y, 7));
                    i64::from(a[2 + k])
                };
                log.extend(l);
                eax
            }
        };
        assert_eq!(got, want_eax, "is_seen — eax, row `{line}`");
        assert_eq!(log, want_log, "is_seen — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_seen — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_seen — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "is_seen — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ItemData::is_seen@00677850: {n} rows agree");
}
