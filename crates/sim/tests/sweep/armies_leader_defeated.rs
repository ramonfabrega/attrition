//! `Armies::leader_defeated@006f2f90` under the emulator against the port (item 2309; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Armies::leader_defeated@006f2f90`, run by `tools/emu/sweep_obj.py` on its row `armies_leader_defeated` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 1 argument(s), 224 random bytes at 0x00c09700; callee 0x006f9180 (0 args) stubbed; the word at `c0+0x10+a[0]*0x1c` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 4 random bytes less `0` (placed); the word at `p0+4` points at 4 random bytes less `0` (placed); the word at `p0+8` points at 4 random bytes less `0` (placed); the word at `p0+0xc` points at 4 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("armies_leader_defeated") else {
        return;
    };
    let obj = super::Obj {
        name: "leader_defeated",
        full: "Armies::leader_defeated@006f2f90",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Leader a0's armies (the armies table at 0xc09700, 0x1c a row: the
            // count +4, the list +0x10): each live army (its first short) stopped.
            let n = rd(&c[0], 4 + 0x1c * a[0] as usize);
            let calls: Vec<String> = (0..n.max(0) as usize)
                .filter(|&k| p[1 + k][0] != 0 || p[1 + k][1] != 0)
                .map(|k| format!("p{}+0x0", 1 + k))
                .collect();
            log.push(if calls.is_empty() {
                "-".into()
            } else {
                calls.join("/")
            });
            0
        }
    });
}
