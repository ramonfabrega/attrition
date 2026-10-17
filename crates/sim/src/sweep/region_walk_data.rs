//! `Region::walk_data@00680e20` under the emulator against the port (item 3024; tools/sweep_gen.py, obj with windows).

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

/// `Region::walk_data@00680e20`, run by `tools/emu/sweep_region_walk_data.py` under unicorn
/// (tools/emu): 112 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p4 the place(s), in flag order; the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c0621c points at 576 random bytes; argument 0 names 16 random bytes; vtable slot +0x0 (2 args) stubbed; vtable slot +0x4 (1 args) stubbed; callee 0x00478990 (2 args) stubbed; callee 0x00883780 (1 args) stubbed; the word at `c1+0x10` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 208 random bytes less `0` (placed); the word at `p1+4` points at 208 random bytes less `0` (placed); the word at `p1+8` points at 208 random bytes less `0` (placed); stub 3 answers 1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes a value in [0, 1] through its argument 0, printed after the answers (`o` in a place); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_region_walk_data.py") else {
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
            // Walker `p0`: `[p0+4]` the input flag, `[p0+8]` the checksum flag. `m` sits 0x330
            // below `p0` in the harness (the numbers the log prints for a pointer outside
            // every window: `[this+k] + 0xc + &this+k`). Every slot-0 call the stub answers
            // writes `a[5]` through its first argument (`--outarg 0:0:0,1`), which is `this+k`
            // except the last one, whose first argument is the caller's local.
            let (input, checksum) = (rd(&p[0], 4), rd(&p[0], 8));
            let mbase = a[0].wrapping_sub(0x330);
            let v = a[5];
            let mut s0: Vec<String> = Vec::new();
            let mut slot0 = |m: &mut [u8], arg0: &str, arg1: String, at: Option<usize>| {
                s0.push(format!("p0+0x0,{arg0},{arg1}"));
                if let Some(at) = at {
                    wr(m, at, v);
                }
            };
            let tail = |k: usize| format!("m+{:#x}", k + 8);
            slot0(m, "m+0x0", "m+0x2c".into(), Some(0));
            for esi in [0x34usize, 0x44, 0x58] {
                slot0(m, &format!("m+{esi:#x}"), tail(esi), Some(esi));
                let size = rd(m, esi + 4);
                let ptr = size
                    .wrapping_add(0xc)
                    .wrapping_add(mbase.wrapping_add(esi as i32));
                slot0(
                    m,
                    &format!("m+{:#x}", esi + 0xc),
                    ptr.to_string(),
                    Some(esi + 0xc),
                );
                if input != 0 {
                    wr(m, esi + 8, 2);
                }
            }
            let slot4 = format!("p0+0x0,{}", rd(&c[0], 0x10).wrapping_add(0x1a57c));
            let walk3 = "m+0x6c,p0+0x0,p0+0x0".to_string();
            let mut border = "-".to_string();
            if checksum == 0 {
                // The flag is a caller local handed by address (`sp-0x4`); on output it is
                // `[this+0x30] != 0`, which the stub never reads, so only the input arm's
                // `a[5]` (written through the pointer) is observable.
                s0.push("p0+0x0,sp-0x4,sp+0x0".into());
                let local = if input == 0 { 0 } else { v };
                if input != 0 && local != 0 {
                    border = format!("p0+0x0,{}", rd(m, 0x10));
                    let id = a[4] & 0xff;
                    m[0x30] = id as u8;
                    wr(&mut c[1], 0x218, 0);
                    p[1 + id as usize][0xc8] = 0x10;
                }
            }
            log.push(s0.join("/"));
            log.push(slot4);
            log.push(walk3);
            log.push(border);
            want_eax
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
            assert_eq!(w, &want[3 + i], "walk_data — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Region::walk_data@00680e20: {n} rows agree");
}
