//! `project@0092d000` under the emulator against the port (item 2818; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `project@0092d000`, run by `tools/emu/sweep_obj.py` on its row `project_d` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 4 argument(s), argument 2 names 4 random bytes; argument 3 names 4 random bytes; callee 0x00a46a00 (0 args) stubbed; `__cdecl`: the caller pops the arguments; stub 0 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; `ecx` an integer in -2147483648..2147483647, not the object, the row's last; `edx` an integer in -2147483648..2147483647, after `ecx` on the row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("project_d") else {
        return;
    };
    let obj = super::Obj {
        name: "project",
        full: "project@0092d000",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // `project`: sin_table's two (ecx, edx) pairs, the quadrant folded: a negative angle negates the
            // distance and drops its sign bit, bit 30 reflects it (0x7fffffff - e); the second call is 90 degrees on.
            // A zero distance calls nothing and the base comes back as it was.
            let fold = |angle: i32, dist: i32| -> (i32, i32) {
                let (mut e, mut d) = (angle, dist);
                if e < 0 {
                    d = d.wrapping_neg();
                    e &= 0x7fff_ffff;
                }
                (
                    if e & 0x4000_0000 != 0 {
                        0x7fff_ffff - e
                    } else {
                        e
                    },
                    d,
                )
            };
            let (ang, dist, bx, by) = (a[0], a[1], a[5], a[6]);
            let ans = a[4];
            let mut calls: Vec<String> = Vec::new();
            let x = if dist == 0 {
                bx
            } else {
                let (e, d) = fold(ang, dist);
                calls.push(format!("{e},{d}"));
                bx.wrapping_add(ans)
            };
            let y = if dist == 0 {
                by
            } else {
                let (e, d) = fold(ang.wrapping_add(0x4000_0000), dist);
                calls.push(format!("{e},{d}"));
                by.wrapping_sub(ans)
            };
            log.push(if calls.is_empty() {
                "-".to_string()
            } else {
                calls.join("/")
            });
            wr(&mut p[0], 0, x);
            wr(&mut p[1], 0, y);
            i64::from(a[3])
        }
    });
}
