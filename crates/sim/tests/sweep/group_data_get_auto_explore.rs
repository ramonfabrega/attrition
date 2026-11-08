//! `GroupData::get_auto_explore@0070c1b0` under the emulator against the port (item 2782; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GroupData::get_auto_explore@0070c1b0`, run by `tools/emu/sweep_obj.py` on its row `group_data_get_auto_explore` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2508 random bytes of object, 1 argument(s), 8 random bytes at 0x00ee1510; argument 0 names 4 random bytes (null a third of the rows); callee 0x0070d000 (0 args) stubbed; callee 0x0070ccb0 (1 args) stubbed; vtable slot +0x18 (0 args) stubbed; the word at `0xc0ab84+m[0x4a]*0x1c` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 32 random bytes less `0` (placed); the word at `p1+4` points at 32 random bytes less `0` (placed); the word at `p1+8` points at 32 random bytes less `0` (placed); the word at `p1+12` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+m[0x4a]*0x1c` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 112 random bytes less `0` (placed); the word at `p6+4` points at 112 random bytes less `0` (placed); the word at `p6+8` points at 112 random bytes less `0` (placed); the word at `p6+12` points at 112 random bytes less `0` (placed); stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 2 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 0 answers 0..3 every row; stub 1 answers -1..3 every row; stub 2 answers 0..1 every row; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_data_get_auto_explore") else {
        return;
    };
    let obj = super::Obj {
        name: "get_auto_explore",
        full: "GroupData::get_auto_explore@0070c1b0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // `buildings` answers 0 with `*out = 1`. Else `get_num_const` (stub 0, a[4..]) under two is the
            // leader arm: `num > 0`, the leader (stub 1, a[12]) in range and its object's flag +8 set, then
            // bit 8 of the unit's word +0x68; any other answers 0 with `*out = 1`. Over one it zeroes the image's
            // `counts`, walks the entries `get_num_const` says (asked again after each), counts those whose
            // slot +0x18 (stub 2, a[14..]) answers by the unit's bit 8, and answers 1 when counts[1] > 0 and
            // counts[0] <= counts[1]; `*out` is then `bVar5 < 2`, bVar5 = (counts[0] > 0) + (counts[1] > 0).
            let (mut c0, mut c2) = (0usize, 0usize);
            let list = |k: usize| {
                usize::try_from(i16::from_le_bytes([m[0x8cc + 2 * k], m[0x8cd + 2 * k]]))
                    .expect("an index")
            };
            let bit8 = |u: &[u8]| usize::from(u[0x69] & 1);
            let (res, outv): (i64, i32) = if m[0x49] != 0 {
                (0, 1)
            } else {
                let n = a[4 + c0];
                c0 += 1;
                if n < 2 {
                    let leader = a[12];
                    if rd(m, 12) < 1
                        || leader < 0
                        || p[2 + usize::try_from(leader).expect("an index")][8] & 1 == 0
                    {
                        (0, 1)
                    } else {
                        (
                            i64::from(p[7 + usize::try_from(leader).expect("an index")][0x69] & 1),
                            1,
                        )
                    }
                } else {
                    let mut cnt = [0i32; 2];
                    let limit0 = a[4 + c0];
                    c0 += 1;
                    let mut b5 = 0i32;
                    let mut c1v = 0i32;
                    if limit0 >= 1 {
                        let mut limit = limit0;
                        let mut i = 0usize;
                        while (i as i32) < limit {
                            let idx = list(i);
                            if a[14 + c2] != 0 {
                                cnt[bit8(&p[7 + idx])] += 1;
                            }
                            c2 += 1;
                            i += 1;
                            limit = a[4 + c0];
                            c0 += 1;
                        }
                        b5 = i32::from(cnt[0] > 0);
                        c1v = cnt[1];
                    }
                    let mut u4 = 0i64;
                    if c1v > 0 {
                        b5 += 1;
                        if cnt[0] <= c1v {
                            u4 = 1;
                        }
                    }
                    c[0][..8].fill(0);
                    wr(&mut c[0][..], 0, cnt[0]);
                    wr(&mut c[0][..], 4, cnt[1]);
                    (u4, i32::from(b5 < 2))
                }
            };
            if a[0] != 0 {
                wr(&mut p[0][..], 0, outv);
            }
            res
        }
    });
}
