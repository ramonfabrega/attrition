//! `ResourceDivvyPool::add_good@0068a830` under the emulator against the port (item 2946; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ResourceDivvyPool::add_good@0068a830`, run by `tools/emu/sweep_obj.py` on its row `resource_divvy_pool_add_good` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 128 random bytes of object, 1 argument(s), vtable slot +0xe8 (0 args) stubbed; vtable slot +0xec (0 args) stubbed; vtable slot +0x0 (1 args) stubbed; the word at `0xc061ac` points at 32 random bytes less `0` (placed); the word at `p0+0x10` points at 32 random bytes less `0` (placed); the word at `p1+0` points at 768 random bytes less `0` (placed); the word at `0xe3a380` points at 736 random bytes less `0` (placed); the word at `m+0x1c` points at 64 random bytes less `0` (placed); the word at `m+0x44` points at 64 random bytes less `0` (placed); the word at `m+0x6c` points at 64 random bytes less `0` (placed); the word at `p2+0` laid as the image vtable at 0xb44b70 (the stub slots copied there) half the rows; the word at `p1+4` laid as the address of `p2` every row; the word at `p1+8` laid as the address of `p2` every row; the word at `p1+12` laid as the address of `p2` every row; the word at `p1+16` laid as the address of `p2` every row; the word at `p1+20` laid as the address of `p2` every row; the word at `p1+24` laid as the address of `p2` every row; the word at `p1+28` laid as the address of `p2` every row; stub 0 answers 0..5 every row; stub 1 answers 0..5 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `m+0xc` as well as at `this`; the stubbed vtable laid at `m+0x34` as well as at `this`; the stubbed vtable laid at `m+0x5c` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("resource_divvy_pool_add_good") else {
        return;
    };
    let obj = super::Obj {
        name: "add_good",
        full: "ResourceDivvyPool::add_good@0068a830",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // add_good: the good's age (the type record's +0x278 when it is the base class's and set, else a vtable slot)
        // files it on the late list from age 3, the early list below; a good a land lists as water goes on the water
        // list instead (the six, nine more times). A list at capacity is asked to grow first (its slot 0, handed the
        // list's step), which the sweep's stub does not do.
        let vftable = rd(&p[2], 0) == 0xb44b70u32 as i32;
        let age = if vftable {
            let v = rd(&p[2], 0x278);
            if v == -1 { a[1] } else { v }
        } else {
            a[2]
        };
        let push = |m: &mut [u8],
                    p: &mut [Vec<u8>],
                    base: usize,
                    win: usize,
                    v: i32,
                    log: &mut Vec<String>| {
            let count = rd(m, base + 4);
            if count >= rd(m, base + 8) {
                log.push(format!(
                    "m+{:#x},{}",
                    base,
                    i16::from_le_bytes([m[base + 12], m[base + 13]])
                ));
            }
            wr(&mut p[win], 4 * count as usize, v);
            wr(m, base + 4, count + 1);
        };
        let lands = (0..rd(&p[3], 0x2bc)).any(|i| rd(&p[3], 0x2c0 + 4 * i as usize) == a[0]);
        if lands {
            push(m, p, 0x5c, 6, a[0], &mut *log);
            if a[0] == 6 {
                for _ in 0..9 {
                    push(m, p, 0x5c, 6, 6, &mut *log);
                }
            }
        } else if age >= 3 {
            push(m, p, 0x34, 5, a[0], &mut *log);
        } else {
            push(m, p, 0xc, 4, a[0], &mut *log);
        }
        if log.is_empty() {
            log.push("-".to_string());
        }
        let joined = log.join("/");
        log.clear();
        log.push(joined);
        0
    });
}
