//! `AmmoData::walk_data@0067ab50` under the emulator against the port (item 3058; tools/sweep_gen.py, obj with windows).

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

/// `AmmoData::walk_data@0067ab50`, run by `tools/emu/sweep_ammo_data_walk_data.py` under unicorn
/// (tools/emu): 112 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; the global cell 0x00c06378 points at 32 random bytes; argument 0 names 16 random bytes; vtable slot +0x0 (2 args) stubbed; vtable slot +0x4 (1 args) stubbed; callee 0x00478360 (0 args) stubbed; callee 0x00478310 (0 args) stubbed; callee 0x009132b0 (1 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes a value in [0, 1, 2, 3, 4, 256, 257, 5, 16, 48] through its argument 0, printed after the answers (`o` in a place); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_ammo_data_walk_data.py") else {
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
            {
                // `a`: 0 the DataWalk pointer, 1..=5 the stubs' answers (a[3] what the recycler pops), 6 the value
                // slot +0 writes through its first argument on every call: `flags` is the dword at this+4, `rolling`
                // the dword at this+5, and the walked byte is a stack local (only its low byte is read).
                let v = a[6];
                let input = rd(&p[0], 4);
                let ammo = |m: &[u8]| rd(m, 0x68);
                let mut l0 = vec!["p0+0x0,m+0x4,m+0x5".to_string()];
                let mut l3 = "-".to_string();
                let mut l4 = "-".to_string();
                let l1 = format!("p0+0x0,{}", rd(&c[0], 0x10).wrapping_add(0x9d8));
                let mut l2 = "-".to_string();
                wr(m, 4, v);
                if m[4] & 3 != 0 {
                    l0.push("p0+0x0,m+0x5,m+0x68".to_string());
                    wr(m, 5, v);
                }
                l0.push("p0+0x0,sp-0x1,sp+0x0".to_string());
                let local = (v & 0xff) as u8;
                let _ = ammo(m);
                if input == 0 {
                    if local != 0 {
                        l4 = format!("{},p0+0x0", ammo(m));
                    }
                } else if local == 0 {
                    if ammo(m) != 0 {
                        l3 = format!("{}", ammo(m).wrapping_add(0x3c));
                        wr(m, 0x68, 0);
                    }
                } else if ammo(m) == 0 {
                    l2 = "p0+0x0".to_string();
                    wr(m, 0x68, a[3]);
                    l4 = format!("{},p0+0x0", a[3]);
                } else {
                    l4 = format!("{},p0+0x0", ammo(m));
                }
                log.extend([l0.join("/"), l1, l2, l3, l4]);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "walk_data — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "walk_data — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "walk_data — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "walk_data — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("AmmoData::walk_data@0067ab50: {n} rows agree");
}
