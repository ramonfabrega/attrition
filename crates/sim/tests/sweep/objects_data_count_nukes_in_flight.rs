//! `ObjectsData::count_nukes_in_flight@00658490` under the emulator against the port (item 2837; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectsData::count_nukes_in_flight@00658490`, run by `tools/emu/sweep_obj.py` on its row `objects_data_count_nukes_in_flight` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `0xc0aca8` points at 32 random bytes less `0` (placed); the word at `p0+0x10` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 68 random bytes less `0` (placed); the word at `p1+4` points at 68 random bytes less `0` (placed); the word at `p1+8` points at 68 random bytes less `0` (placed); the word at `p1+12` points at 68 random bytes less `0` (placed); the word at `0xc0ab84` points at 8 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 8 random bytes less `0` (placed); the word at `p6+0` points at 32 random bytes less `0` (placed); the word at `p6+4` points at 32 random bytes less `0` (placed); the word at `p7+0` points at 32 random bytes less `0` (placed); the word at `p7+4` points at 32 random bytes less `0` (placed); the word at `p8+0x18` points at 496 random bytes less `0` (placed); the word at `p9+0x18` points at 496 random bytes less `0` (placed); the word at `p10+0x18` points at 496 random bytes less `0` (placed); the word at `p11+0x18` points at 496 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("objects_data_count_nukes_in_flight") else {
        return;
    };
    let obj = super::Obj {
        name: "count_nukes_in_flight",
        full: "ObjectsData::count_nukes_in_flight@00658490",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // The list (window p0, count at `+4`, objects behind it, p2..p5) holds the objects whose
        // `+4` flags have bits 0 or 1 and whose table (`+0x3c`) and index (`+0x40`) are not
        // negative: the object they name (`objects[who][o]`, p8 + 2·who + o) whose type (`+0x18`,
        // p12 + 2·who + o) has bit 27 of `+0x1e4` is asked `is(0x13b, 0)`, through its own slot
        // +0xb8 (stub 0, `a[0]`) or, laid as `ObjectData::is` (`a[2]`), its type's slot +0x60
        // (stub 1, `a[1]`); a yes counts (item 2837).
        let mut n = 0;
        for e in 0..usize::try_from(rd(&p[0], 4).max(0)).expect("a count") {
            let o = &p[2 + e];
            let (who, ix) = (rd(o, 0x3c), rd(o, 0x40));
            if rd(o, 4) & 3 == 0 || ix < 0 || who < 0 {
                continue;
            }
            let at = (2 * who + ix) as usize;
            if rd(&p[12 + at], 0x1e4) & 0x8000000 == 0 {
                continue;
            }
            let yes = if a[2] != 0 {
                log.push(format!("s1:p{}+0x0,315,0", 12 + at));
                a[1]
            } else {
                log.push(format!("s0:p{}+0x0,315,0", 8 + at));
                a[0]
            };
            if yes != 0 {
                n += 1;
            }
        }
        if log.is_empty() {
            log.push("-".into());
        }
        n
    });
}
