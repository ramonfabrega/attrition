//! `Objects::add_ammo@00658b10` under the emulator against the port (item 2400; tools/sweep_gen.py, obj with windows).

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

/// `Objects::add_ammo@00658b10`, run by `tools/emu/sweep_objects_add_ammo.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), the global cell 0x00c0618c points at 512 random bytes; argument 0 names 16 random bytes; vtable slot +0x0 (1 args) stubbed; the import cell 0x00ac54f0 holds a stub (1 args); callee 0x0055d1b3 (5 args) stubbed; callee 0x00422300 (1 args) stubbed; callee 0x0067bbf0 (3 args) stubbed; the word at `c0+0x12c` points at 64 random bytes less `0` (placed); the word at `p1+0x0` points at 8 random bytes less `0` (placed); the word at `p1+0x4` points at 8 random bytes less `0` (placed); the word at `p1+0x8` points at 8 random bytes less `0` (placed); the word at `p1+0xc` points at 8 random bytes less `0` (placed); the word at `p1+0x10` points at 8 random bytes less `0` (placed); the word at `p1+0x14` points at 8 random bytes less `0` (placed); the word at `p1+0x18` points at 8 random bytes less `0` (placed); the word at `p1+0x1c` points at 8 random bytes less `0` (placed); stub 1 answers the address of 128 random bytes, a `p` window after the places; the stubbed vtable laid at `c0+0x11c`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 1 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_add_ammo.py") else {
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
                // The pool (`c0+0x12c` -> p1, `c0+0x120` entries, each `p(2+i)`), whose
                // entries in use (`flags & 3`, `+4`; every one, not a prefix: a gap is found) are the slots `lowest_free_slot`
                // is handed; a full pool is grown by one `Ammo` (malloc, stub 1's
                // answer `a[2]`, handed `+4`), and `Ammo::init` is run on the slot.
                let n = rd(&c[0], 0x120);
                let live: Vec<u32> = (0..n.max(0) as usize)
                    .filter(|i| p[2 + i][4] & 3 != 0)
                    .map(|i| i as u32)
                    .collect();
                let slot = crate::fight::lowest_free_slot(&live) as usize;
                let at = if slot as i32 == n {
                    log.push("s0:c0+0x11c,1".into());
                    let ptr = a[2].wrapping_add(4);
                    log.push("s2:c0+0x11c,p10+0x4,120,1,6797104,6797088".into());
                    wr(&mut p[1], 4 * slot, ptr);
                    wr(&mut p[10], 0, 1); // the array-new count word
                    let cap = rd(&c[0], 0x124);
                    if cap < slot as i32 + 1 {
                        let diff = slot as i32 + 1 - cap;
                        let s = i32::from(i16::from_le_bytes([c[0][0x128], c[0][0x129]]));
                        let local = if s < 0 { cap } else { s };
                        let arg = if diff > local {
                            diff
                        } else if s >= 0 {
                            s
                        } else {
                            cap
                        };
                        log.push(format!("s3:c0+0x11c,{arg}"));
                    }
                    wr(&mut c[0], 0x120, n.max(slot as i32 + 1));
                    "p10+0x4".to_string()
                } else {
                    format!("p{}+0x0", 2 + slot)
                };
                let counter = rd(&c[0], 0x1f8);
                log.push(format!("s4:{at},p0+0x0,{slot},{counter}"));
                wr(&mut c[0], 0x1f8, counter.wrapping_add(1));
                slot as i64
            }
        };
        assert_eq!(got, want_eax, "add_ammo — eax, row `{line}`");
        assert_eq!(log, want_log, "add_ammo — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_ammo — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "add_ammo — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "add_ammo — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 500, "only {n} rows");
    eprintln!("Objects::add_ammo@00658b10: {n} rows agree");
}
