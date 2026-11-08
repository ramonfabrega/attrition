//! `Type::backup@00667ee0` under the emulator against the port (item 3064; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Type::backup@00667ee0`, run by `tools/emu/sweep_obj.py` on its row `type_backup` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p2 the place(s), in flag order, p3 stub 0's answer window; argument 0 names 768 random bytes; callee 0x00a1f0a0 (0 args) stubbed; callee 0x0055e0ac (3 args) stubbed; the word at `m+0x74` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `m+0xb0` points at 16 random bytes less `0` (placed, null a third of the rows); stub 0 answers the address of 512 random bytes, a `p` window after the places; stub 1 is `__cdecl`: the caller pops its arguments; stub 1 copies its argument 2 bytes from argument 1 to argument 0 (a `memcpy`); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("type_backup") else {
        return;
    };
    let obj = super::Obj {
        name: "backup",
        full: "Type::backup@00667ee0",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // `a[0]` the backup record `p[0]`: the job time, type and tribe mask copied from `m+8`, `m+4`, `m+0x10`;
        // the name (string at `m+0x74`, its list head `p[1]`, length short at `m+0x7c`, at most 99 characters) and the
        // type name (`m+0xb0`, `p[2]`, `m+0xb8`, at most 255) are copied from `String::str`'s answer `p[3]`
        // to `+0x20c` and `+0xc` (2n+2 bytes, the terminator written after the n characters), or cleared when
        // the string, its head or its length is null; then three prerequisites and six costs.
        let (head, tn) = (rd(&p[1], 0), rd(&p[2], 0));
        let copy_name = |p: &mut [Vec<u8>], at: usize, ptr: i32, h: i32, len: u16, cap: usize| {
            if ptr != 0 && h != 0 && len != 0 {
                let n = (len as usize).min(cap);
                let src = p[3][..2 * n + 2].to_vec();
                p[0][at..at + 2 * n + 2].copy_from_slice(&src);
                p[0][at + 2 * n..at + 2 * n + 2].fill(0);
            } else {
                p[0][at..at + 2].fill(0);
            }
        };
        let w = rd(m, 8);
        wr(&mut p[0], 0, w);
        let w = rd(m, 0x10);
        wr(&mut p[0], 8, w);
        let w = rd(m, 4);
        wr(&mut p[0], 4, w);
        let name_len = u16::from_le_bytes([m[0x7c], m[0x7d]]);
        copy_name(p, 0x20c, rd(m, 0x74), head, name_len, 0x63);
        let type_len = u16::from_le_bytes([m[0xb8], m[0xb9]]);
        copy_name(p, 0xc, rd(m, 0xb0), tn, type_len, 0xff);
        for (to, from) in [
            (0x2d4, 0x30),
            (0x2d8, 0x34),
            (0x2dc, 0x38),
            (0x2e0, 0x18),
            (0x2e4, 0x1c),
            (0x2e8, 0x20),
            (0x2ec, 0x24),
            (0x2f0, 0x28),
            (0x2f4, 0x2c),
        ] {
            let w = rd(m, from);
            wr(&mut p[0], to, w);
        }
        0
    });
}
