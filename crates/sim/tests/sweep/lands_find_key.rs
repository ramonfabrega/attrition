//! `Lands::find_key@0067e610` under the emulator against the port (item 2346; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Lands::find_key@0067e610`, run by `tools/emu/sweep_obj.py` on its row `lands_find_key` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 2 argument(s), the global cell 0x00e3a380 points at 2808 random bytes; the global cell 0x00c06378 points at 32 random bytes; argument 0 names 16 random bytes; callee 0x00a1ba80 (2 args) stubbed; the word at `c1+0x10` points at 16 random bytes less `0x167ec` (placed); stub 0 answered from [0, 0, 0, 0, 0, 0, 0, 0, 1] (90%); stub 0 answers 10 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("lands_find_key") else {
        return;
    };
    let obj = super::Obj {
        name: "find_key",
        full: "Lands::find_key@0067e610",
        cells: 2,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The key (argument 0, a `String`) against the nine lands' names (+0x11c of each 0x138-byte
            // row of the table cell 0xe3a380 names) by the string compare `ignore(name, -1)` (stub 0), the first match's
            // index answered; none → the same against the int-string table's +0x167ec, and -1.
            let mut calls = Vec::new();
            let mut found = -1;
            for i in 0..9 {
                calls.push(format!("p0+0x0,c0+{:#x},-1", 0x11c + i * 0x138));
                if a[3 + i] != 0 {
                    found = i as i64;
                    break;
                }
            }
            if found < 0 {
                calls.push("p0+0x0,p1+0x0,-1".into());
            }
            log.push(calls.join("/"));
            found
        }
    });
}
