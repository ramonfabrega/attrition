//! `Unit::cavarch_fight@005ff4b0` under the emulator against the port (item 3134; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::cavarch_fight@005ff4b0`, run by `tools/emu/sweep_obj.py` on its row `unit_cavarch_fight` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 176 random bytes of object, 0 argument(s), `p` windows in order: p0..p8 the place(s), in flag order; the global cell 0x00c0618c points at 96 random bytes; callee 0x005fd4d0 (5 args) stubbed; vtable slot +0xe4 (0 args) stubbed; the word at `m+0x18` points at 736 random bytes less `0` (placed); the word at `c0+0x14` points at 12 random bytes less `0` (placed); the word at `c0+0x30` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 32 random bytes less `0` (placed); the word at `p1+4` points at 32 random bytes less `0` (placed); the word at `p1+8` points at 32 random bytes less `0` (placed); the word at `p2+0` points at 32 random bytes less `0` (placed); the word at `p2+4` points at 32 random bytes less `0` (placed); the word at `p2+8` points at 32 random bytes less `0` (placed); stub 1 answered from [0, 1, 2, -1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_cavarch_fight") else {
        return;
    };
    let obj = super::Obj {
        name: "cavarch_fight",
        full: "Unit::cavarch_fight@005ff4b0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 1200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let rd = |w: &[u8], o: usize| {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            };
            let rd16 = |w: &[u8], o: usize| {
                i16::from_le_bytes(w[o..o + 2].try_into().expect("two bytes")) as i32
            };
            // a[1] the slot +0xe4 stub's answer; windows: m the unit, p0 its object record (+0x18), p3 + 3 * player + index
            // the units a player's array names. `fight` is stubbed; FIGHT_WRITES says whether it leaves the target as (1, 0).
            const FIGHT_WRITES: bool = false;
            let (idx, pl) = (rd16(m, 0xa2), m[0xa8] as i8 as i32);
            if rd(&p[0], 0x2b4) & 0x400 != 0 {
                let v = rd(&p[0], 0x2d0);
                p[0][0x1fc..0x200].copy_from_slice(&v.to_le_bytes());
            }
            log.push(format!("m+0x0,{},{},{},0,1", idx, pl, rd(m, 0x6c) & 0x100));
            if FIGHT_WRITES {
                m[0xa2..0xa4].copy_from_slice(&1i16.to_le_bytes());
                m[0xa8] = 0;
            }
            if rd(&p[0], 0x2b4) & 0x400 != 0 {
                p[0][0x1fc..0x200].copy_from_slice(&0i32.to_le_bytes());
            }
            if idx >= 0
                && pl >= 0
                && (rd16(m, 0xa2) != idx || m[0xa8] as i8 as i32 != pl)
                && (m[0xa8] as i8 as i32 != pl || a[1] != rd16(m, 0xa2))
            {
                let v = rd(m, 0x6c) & !0x100;
                m[0x6c..0x70].copy_from_slice(&v.to_le_bytes());
            }
            if rd16(m, 0xa2) < 0 && (idx as u16) >= 0x8000 {
                m[0xa2..0xa4].copy_from_slice(&(-2i16).to_le_bytes());
            }
            0
        }
    });
}
