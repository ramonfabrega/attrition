//! `Object::adjacent_to@00651f40` under the emulator against the port (item 2368; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Object::adjacent_to@00651f40`, run by `tools/emu/sweep_obj.py` on its row `object_adjacent_to` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00c0618c points at 256 random bytes; the global cell 0x00c061f0 points at 256 random bytes; vtable slot +0x18 (0 args) stubbed; callee 0x0064c880 (2 args) stubbed; callee 0x006488f0 (4 args) stubbed; the word at `c0+0x14+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p0+4*a[0]` points at 16 random bytes less `0` (placed); the word at `m+0x18` points at 544 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*m[9]` points at 32 random bytes less `0` (placed); the word at `p3+4*i16(m,0xa)` points at 32 random bytes less `0` (placed); the word at `p4+0x18` points at 696 random bytes less `0` (placed); stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 99, 100, 575, 576, 577, 483, 484, 485, 959, 960, 961, 2000] (90%); stub 2 answered from [0, 95, 96, 97, 1000] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("object_adjacent_to") else {
        return;
    };
    let obj = super::Obj {
        name: "adjacent_to",
        full: "Object::adjacent_to@00651f40",
        cells: 2,
        calls: true,
        eax: true,
        min_rows: 1500,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `a[0]` the target's id, `a[1]` its player; the answers from
        // `a[2]`: `+0x18` (is a unit), the boat arm's `attack_dist`, the
        // generic arm's `attack_dist`. This's slot is `m` (`+8` in use,
        // `+0x10`/`+0x14` the masked position), the target's `p[1]`
        // (`+8`), this's type `p[2]` (`+0x218` the domain), the unit's
        // own type read back through the units table `p[5]` (`+0x2b4`
        // the flags, `+0x248` `coll_size`), the constants `c[1]`
        // (`+0x94`).
        let (x, y) = (rd(m, 0x10) ^ 0x63637, rd(m, 0x14) ^ 0x63637);
        let (p2, p5) = (&p[2], &p[5]);
        let ok = crate::orders::adjacent_steps(
            m[8] & 1 != 0,
            p[1][8] & 1 != 0,
            rd(p2, 0x218) == 1,
            || a[2] != 0,
            || (p5[0x2b4] & 0x10 != 0, rd(p5, 0x248)),
            rd(&c[1], 0x94),
            |boat| {
                if boat {
                    log.push(format!("s1:m+0x0,{},{}", a[0], a[1]));
                    a[3]
                } else {
                    log.push(format!("s2:m+0x0,{},{},{x},{y}", a[0], a[1]));
                    a[4]
                }
            },
        );
        if log.is_empty() {
            log.push("-".into());
        }
        i64::from(ok)
    });
}
