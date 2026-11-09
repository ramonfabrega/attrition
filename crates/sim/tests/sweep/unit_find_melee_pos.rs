//! `Unit::find_melee_pos@006010b0` under the emulator against the port (item 3289; tools/sweep_gen.py, obj with windows).

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

/// `Unit::find_melee_pos@006010b0`, run by `tools/emu/sweep_unit_find_melee_pos.py` under unicorn
/// (tools/emu): 32 random bytes of object, 8 argument(s), `p` windows in order: p0..p1 pointed argument(s), p2..p7 the place(s), in flag order, p8..p13 the stubs' answer window(s); the global cell 0x00c0618c points at 256 random bytes; the global cell 0x00c06188 points at 64 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; argument 3 names 4 random bytes; argument 4 names 4 random bytes; vtable slot +0xb8 (2 args) stubbed; callee 0x00607c30 (8 args) stubbed; callee 0x0065b1b0 (5 args) stubbed; callee 0x0065b440 (4 args) stubbed; the import cell 0x00ac54f0 holds a stub (1 args); callee 0x0055e0ac (3 args) stubbed; the import cell 0x00ac5500 holds a stub (1 args); the word at `0xc0aec0+0x1c*a[1]` points at 32 random bytes less `0` (placed); the word at `p2+4*a[0]` points at 256 random bytes less `0` (placed); the word at `p3+0x18` points at 608 random bytes less `0` (placed); the word at `m+0x18` points at 608 random bytes less `0` (placed); the word at `c0+0x14+0x1c*a[1]` points at 32 random bytes less `0` (placed); the word at `p6+4*a[0]` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 0, 0, 0, 5] (90%); stub 2 answered from [0, 0, 0, 0, 1] (90%); stub 3 answered from [0, 0, 0, 0, 1] (90%); stub 4 answers a window of its own to each call, in call order (1024 bytes, ...); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 4 is `__cdecl`: the caller pops its arguments; stub 5 is `__cdecl`: the caller pops its arguments; stub 6 is `__cdecl`: the caller pops its arguments; stub 5 copies its argument 2 bytes from argument 1 to argument 0 (a `memcpy`); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_find_melee_pos.py") else {
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
            // `a[0]` the target's id, `a[1]` its list, `a[3]`/`a[4]` the two
            // out-parameters (`p[0]`, `p[1]`), `a[5]`/`a[6]` the approach
            // point; `a[8]` the `is` slot's answer, `a[9]`..`a[11]` the
            // three predicates'. The asker's type is `p[5]`, the target's
            // `p[4]`, its unit record `p[3]`, its object record `p[7]`
            // (`+0x10`/`+0x14` the masked position), the world `c[1]`.
            let centre = crate::collide::ucell(crate::world::Pos::new(
                rd(&p[7], 0x10) ^ 0x63637,
                rd(&p[7], 0x14) ^ 0x63637,
            ));
            let dims = (rd(&c[1], 0), rd(&c[1], 4));
            let slots = crate::attack_pos::find_open_slots_steps(
                rd(&p[5], 0x248),
                rd(&p[4], 0x248),
                a[8] != 0,
                centre,
                dims,
                |at, tl| {
                    log.push(format!("s1:m+0x0,{},{},0,0,0,0,1,{}", tl.x, tl.y, tl.y));
                    if a[9] != 0 {
                        return false;
                    }
                    let (sh, by) = (i16::from_le_bytes([m[0xa], m[0xb]]), m[9]);
                    log.push(format!("s2:{},{},{},{sh},{by},0", at.x, at.x, at.y));
                    if a[10] != 0 {
                        return false;
                    }
                    log.push(format!("s3:m+0x0,{},{},{sh},{by}", at.x, at.y));
                    a[11] == 0
                },
            );
            if log.is_empty() {
                log.push("-".into());
            }
            // The local array's heap blocks (`p[8..]`, the `malloc` stub's
            // windows in call order): 4 entries, then doubling; a block
            // holds the entries up to its capacity (the old ones copied in,
            // the rest appended), the last one as many as there are.
            assert!(
                slots.len() <= 128,
                "{} slots outrun the windows",
                slots.len()
            );
            let mut cap = 0usize;
            let mut win = 8usize;
            while cap < slots.len() {
                cap = if cap == 0 { 4 } else { cap * 2 };
                for (i, s) in slots.iter().take(cap).enumerate() {
                    wr(&mut p[win], 8 * i, s.x);
                    wr(&mut p[win], 8 * i + 4, s.y);
                }
                win += 1;
            }
            let from = crate::world::Pos::new(a[5], a[6]);
            match crate::attack_pos::nearest_slot(&slots, from) {
                Some(s) => {
                    wr(&mut p[0], 0, s.x);
                    wr(&mut p[1], 0, s.y);
                    1
                }
                None => {
                    let (tx, ty) = (rd(&p[7], 0x10) ^ 0x63637, rd(&p[7], 0x14) ^ 0x63637);
                    wr(&mut p[0], 0, tx);
                    wr(&mut p[1], 0, ty);
                    // SEAM (`601178`-`6011a3`): the crate keeps no `full`
                    // (the target's unit record `+0xac`, a signed byte),
                    // so the sweep applies the original's `+5` capped at
                    // `0x1e` itself.
                    let f = (p[3][0xac] as i8).wrapping_add(5);
                    p[3][0xac] = if f > 0x1e { 0x1e } else { f as u8 };
                    0
                }
            }
        };
        assert_eq!(got, want_eax, "find_melee_pos — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_melee_pos — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_melee_pos — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_melee_pos — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "find_melee_pos — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("Unit::find_melee_pos@006010b0: {n} rows agree");
}
