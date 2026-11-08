//! `MountainsData::total_gather_size@0089c110` under the emulator against the port (item 3004; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `MountainsData::total_gather_size@0089c110`, run by `tools/emu/sweep_obj.py` on its row `mountains_data_total_gather_size` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p9 the place(s), in flag order, p10..p11 the stubs' answer window(s); callee 0x0089d080 (2 args) stubbed; callee 0x0089cf90 (2 args) stubbed; callee 0x0089d170 (4 args) stubbed; the word at `m+0xcc` points at 16 random bytes less `0` (placed); the word at `m+0x5c` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); the word at `p2+4` points at 16 random bytes less `0` (placed); the word at `p3+4` points at 16 random bytes less `0` (placed); the word at `p4+4` points at 16 random bytes less `0` (placed); the word at `p5+4` points at 16 random bytes less `0` (placed); stub 0 answers the address of 8 random bytes, a `p` window after the places; stub 1 answers the address of 8 random bytes, a `p` window after the places; stub 2 answers 4 values in call order, printed after the answers; stub 2 answers -5000..5000 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("mountains_data_total_gather_size") else {
        return;
    };
    let obj = super::Obj {
        name: "total_gather_size",
        full: "MountainsData::total_gather_size@0089c110",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `a[0]`, `a[1]` are the addresses of the two answer windows `p[10]`, `p[11]`
        // (`mnt_loc_y`, `mnt_loc_x`: the value is the word at the pointer they answer),
        // `a[3..7]` the four `gather_size` answers in call order. Each of the
        // `+0xa4` mountains names a range type (`p[0]`, the `+0xcc` array), whose
        // record `p[2 + type]` (`+0x5c` array `p[1]`) is `gather_size`'s object (`ecx`
        // the record + 4 + the `+0xc` of its `+4`, held at 0) handed (x, y, -1, 0).
        let count = rd(m, 0xa4);
        let mut calls: Vec<String> = Vec::new();
        let mut sum = 0i32;
        for i in 0..count.max(0) as usize {
            let t = rd(&p[0], 4 * i) as usize;
            calls.push(format!(
                "p{}+0x4,{},{},-1,0",
                2 + t,
                rd(&p[11], 0),
                rd(&p[10], 0)
            ));
            sum = sum.wrapping_add(a[3 + i]);
        }
        log.push(if calls.is_empty() {
            "-".to_string()
        } else {
            calls.join("/")
        });
        i64::from(sum)
    });
}
