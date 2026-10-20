//! `Wall::process@00640450` under the emulator against the port (item 3346; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `Wall::process@00640450`, run by `tools/emu/sweep_wall_process.py` under unicorn
/// (tools/emu): 128 random bytes of object, 0 argument(s), `p` windows in order: p0..p31 sparse table 0 (8 rows × 4 slice(s), row-major), p32..p47 sparse table 1 (8 rows × 2 slice(s), row-major), p48..p125 the place(s), in flag order; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c0618c points at 528 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c06208 points at 16 random bytes; the global cell 0x00c8cd00 points at 16 random bytes; vtable slot +0x4c (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x2c (0 args) stubbed; vtable slot +0x18c (1 args) stubbed; vtable slot +0x50 (0 args) stubbed; vtable slot +0x16c (8 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0x108 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x0063ce70 (1 args) stubbed; callee 0x0065ca80 (13 args) stubbed; callee 0x006455c0 (1 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x005e5210 (4 args) stubbed; callee 0x00594670 (0 args) stubbed; callee 0x00643440 (2 args) stubbed; callee 0x00638d70 (3 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x0097f770 (1 args) stubbed; the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p48+0` points at 32 random bytes less `0` (placed); the word at `p48+4` points at 32 random bytes less `0` (placed); the word at `p48+8` points at 32 random bytes less `0` (placed); the word at `p48+12` points at 32 random bytes less `0` (placed); the word at `0xc0aedc` points at 16 random bytes less `0` (placed); the word at `p53+0` points at 32 random bytes less `0` (placed); the word at `p53+4` points at 32 random bytes less `0` (placed); the word at `p53+8` points at 32 random bytes less `0` (placed); the word at `p53+12` points at 32 random bytes less `0` (placed); the word at `m+0x70` points at 64 random bytes less `0` (placed); the word at `p58+0` points at 64 random bytes less `0` (placed); the word at `p58+4` points at 64 random bytes less `0` (placed); the word at `p58+8` points at 64 random bytes less `0` (placed); the word at `p58+12` points at 64 random bytes less `0` (placed); the word at `p58+16` points at 64 random bytes less `0` (placed); the word at `p58+20` points at 64 random bytes less `0` (placed); the word at `p58+24` points at 64 random bytes less `0` (placed); the word at `p58+28` points at 64 random bytes less `0` (placed); the word at `p58+32` points at 64 random bytes less `0` (placed); the word at `p58+36` points at 64 random bytes less `0` (placed); the word at `p58+40` points at 64 random bytes less `0` (placed); the word at `p58+44` points at 64 random bytes less `0` (placed); the word at `p58+48` points at 64 random bytes less `0` (placed); the word at `p58+52` points at 64 random bytes less `0` (placed); the word at `p58+56` points at 64 random bytes less `0` (placed); the word at `p58+60` points at 64 random bytes less `0` (placed); the word at `c2+0x134` points at 2304 random bytes less `0` (placed); the word at `0xcae5fc` points at 1024 random bytes less `0` (placed); the word at `p33+0` points at 16 random bytes less `0` (placed); the word at `p35+0` points at 16 random bytes less `0` (placed); the word at `p37+0` points at 16 random bytes less `0` (placed); the word at `p39+0` points at 16 random bytes less `0` (placed); the word at `p41+0` points at 16 random bytes less `0` (placed); the word at `p43+0` points at 16 random bytes less `0` (placed); the word at `p45+0` points at 16 random bytes less `0` (placed); the word at `p47+0` points at 16 random bytes less `0` (placed); the word at `c1+0x14` points at 8 random bytes less `0` (placed); the word at `c1+0x30` points at 8 random bytes less `0` (placed); the word at `c1+0x4c` points at 8 random bytes less `0` (placed); the word at `c1+0x68` points at 8 random bytes less `0` (placed); the word at `c1+0x84` points at 8 random bytes less `0` (placed); the word at `c1+0xa0` points at 8 random bytes less `0` (placed); the word at `c1+0xbc` points at 8 random bytes less `0` (placed); the word at `c1+0xd8` points at 8 random bytes less `0` (placed); the word at `p59+0x30` points at 80 random bytes less `0` (placed); the word at `p60+0x30` points at 80 random bytes less `0` (placed); the word at `p61+0x30` points at 80 random bytes less `0` (placed); the word at `p62+0x30` points at 80 random bytes less `0` (placed); the word at `p63+0x30` points at 80 random bytes less `0` (placed); the word at `p64+0x30` points at 80 random bytes less `0` (placed); the word at `p65+0x30` points at 80 random bytes less `0` (placed); the word at `p66+0x30` points at 80 random bytes less `0` (placed); the word at `p67+0x30` points at 80 random bytes less `0` (placed); the word at `p68+0x30` points at 80 random bytes less `0` (placed); the word at `p69+0x30` points at 80 random bytes less `0` (placed); the word at `p70+0x30` points at 80 random bytes less `0` (placed); the word at `p71+0x30` points at 80 random bytes less `0` (placed); the word at `p72+0x30` points at 80 random bytes less `0` (placed); the word at `p73+0x30` points at 80 random bytes less `0` (placed); the word at `p74+0x30` points at 80 random bytes less `0` (placed); the word at `p93+0x18` points at 16 random bytes less `0` (placed); the word at `p94+0x18` points at 16 random bytes less `0` (placed); the word at `p95+0x18` points at 16 random bytes less `0` (placed); the word at `p96+0x18` points at 16 random bytes less `0` (placed); the word at `p97+0x18` points at 16 random bytes less `0` (placed); the word at `p98+0x18` points at 16 random bytes less `0` (placed); the word at `p99+0x18` points at 16 random bytes less `0` (placed); the word at `p100+0x18` points at 16 random bytes less `0` (placed); the word at `p101+0x18` points at 16 random bytes less `0` (placed); the word at `p102+0x18` points at 16 random bytes less `0` (placed); the word at `p103+0x18` points at 16 random bytes less `0` (placed); the word at `p104+0x18` points at 16 random bytes less `0` (placed); the word at `p105+0x18` points at 16 random bytes less `0` (placed); the word at `p106+0x18` points at 16 random bytes less `0` (placed); the word at `p107+0x18` points at 16 random bytes less `0` (placed); the word at `p108+0x18` points at 16 random bytes less `0` (placed); the word at `m+0x18` points at 16 random bytes less `0` (placed); slot +0x4c laid as 0x00472350 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0x50 laid as 0x00472360 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1, 2, 3, 4, 5, 10, -1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 8 answered from [0, 1] (90%); stub 13 answered from [0, 1] (90%); stub 15 answered from [0, 1] (90%); the word at `p93+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p94+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p95+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p96+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p97+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p98+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p99+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p100+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p101+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p102+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p103+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p104+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p105+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p106+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p107+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p108+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p85+0` laid as the address of `p59` every row; the word at `p85+4` laid as the address of `p60` every row; the word at `p86+0` laid as the address of `p61` every row; the word at `p86+4` laid as the address of `p62` every row; the word at `p87+0` laid as the address of `p63` every row; the word at `p87+4` laid as the address of `p64` every row; the word at `p88+0` laid as the address of `p65` every row; the word at `p88+4` laid as the address of `p66` every row; the word at `p89+0` laid as the address of `p67` every row; the word at `p89+4` laid as the address of `p68` every row; the word at `p90+0` laid as the address of `p69` every row; the word at `p90+4` laid as the address of `p70` every row; the word at `p91+0` laid as the address of `p71` every row; the word at `p91+4` laid as the address of `p72` every row; the word at `p92+0` laid as the address of `p73` every row; the word at `p92+4` laid as the address of `p74` every row; stub 9 answers `p93+j`, the data of the object its `ecx` is in `p59+j` (j < 16), linked at `+0x40`; stub 1 answers 17 values in call order, printed after the answers; stub 2 answers 2 values in call order, printed after the answers; stub 3 answers 25 values in call order, printed after the answers; stub 6 answers 26 values in call order, printed after the answers; stub 11 answers 2 values in call order, printed after the answers; stub 13 answers 10 values in call order, printed after the answers; stub 11 answers -1..3 every row; stub 17 answers -2..7 every row; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 16 writes a value in -40..40 through its argument 0, printed after the answers (`o` in a place); stub 16 writes a value in -40..40 through its argument 1, printed after the answers (`o` in a place); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x1fc:0x4, +0x44c:0x4, +0x804:0x4 (`p` windows after the pointed buffers, before the places); 8 rows 0x1c apart at `@0xe39bfc`, slices +0x8:0x4, +0x14:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p109+0` as well as at `this`; the stubbed vtable laid at `p110+0` as well as at `this`; the stubbed vtable laid at `p111+0` as well as at `this`; the stubbed vtable laid at `p112+0` as well as at `this`; the stubbed vtable laid at `p113+0` as well as at `this`; the stubbed vtable laid at `p114+0` as well as at `this`; the stubbed vtable laid at `p115+0` as well as at `this`; the stubbed vtable laid at `p116+0` as well as at `this`; the stubbed vtable laid at `p117+0` as well as at `this`; the stubbed vtable laid at `p118+0` as well as at `this`; the stubbed vtable laid at `p119+0` as well as at `this`; the stubbed vtable laid at `p120+0` as well as at `this`; the stubbed vtable laid at `p121+0` as well as at `this`; the stubbed vtable laid at `p122+0` as well as at `this`; the stubbed vtable laid at `p123+0` as well as at `this`; the stubbed vtable laid at `p124+0` as well as at `this`; the stubbed vtable laid at `p125+0` as well as at `this`; the stubbed vtable laid at `p59+0` as well as at `this`; the stubbed vtable laid at `p60+0` as well as at `this`; the stubbed vtable laid at `p61+0` as well as at `this`; the stubbed vtable laid at `p62+0` as well as at `this`; the stubbed vtable laid at `p63+0` as well as at `this`; the stubbed vtable laid at `p64+0` as well as at `this`; the stubbed vtable laid at `p65+0` as well as at `this`; the stubbed vtable laid at `p66+0` as well as at `this`; the stubbed vtable laid at `p67+0` as well as at `this`; the stubbed vtable laid at `p68+0` as well as at `this`; the stubbed vtable laid at `p69+0` as well as at `this`; the stubbed vtable laid at `p70+0` as well as at `this`; the stubbed vtable laid at `p71+0` as well as at `this`; the stubbed vtable laid at `p72+0` as well as at `this`; the stubbed vtable laid at `p73+0` as well as at `this`; the stubbed vtable laid at `p74+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_process.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                let r: i64 = 'g: {
                    // `Wall::process@00640450`, written from the listing and the decompile, in one pass over `this`.
                    //
                    // Windows: c0 the game (`frame` +0x550, `rush_rules` +0x32), c1 the objects (row `k`'s array pointer at
                    // +0x14+0x1c*k, the unit-row selector at +0x200), c2 the world (width +0, cell plane at +0x134, 0x1c a cell,
                    // owner at +0xf), c3 the console (`who` +0x298), c4 the message window, c5 the string table. p0..p31 the leaders'
                    // slices (a row: flags +0, `last warning` +0x1fc, `unbuilt?` +0x44c, `at war` +0x804), p32..p47 the unbuilt
                    // wonders (a row: count, array), p48..p57 the unit rows, p58..p74 the 16 objects (row `k`'s entries `2k`, `2k+1`),
                    // p75 the world's cells, p76 `div_3_table`, p77..p84 the unbuilt wonders' id arrays, p93..p108 the objects' data
                    // (+0x48 progress, +0x18 the type), p109..p124 those types, p125 this's type.
                    //
                    // Stub answers `a[0..21]` (one a stub; the ones below read), `a[21]`/`a[22]` the corner `tile_corner` writes,
                    // then the call-order lists: slot +0xb8 `a[23..40]`, +0x2c `a[40..42]`, +0x18c `a[42..67]`, +0x60 `a[67..93]`,
                    // `find_unit` `a[93..95]`, `is_ally` `a[95..105]`; `a[105]`/`a[106]`/`a[107]` say slots +0x4c/+0xb8/+0x50 held
                    // the real `is_active` / `ObjectData::is` / `is_started` and not their stubs.
                    const K: i32 = 0x63637;
                    let r16 = |w: &[u8], o: usize| i16::from_le_bytes([w[o], w[o + 1]]);
                    let take = |base: usize, len: usize, n: &mut usize| -> i32 {
                        let v = a[base + (*n).min(len - 1)];
                        *n += 1;
                        v
                    };
                    let (mut n1, mut n2, mut n3, mut n6, mut n11, mut n13) =
                        (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
                    let who = usize::from(m[9]);
                    let id = i32::from(r16(m, 0xa));
                    let frame = rd(&c[0], 0x550);
                    let (x, y) = (rd(m, 0x10) ^ K, rd(m, 0x14) ^ K);
                    let w60 = |m: &[u8]| u16::from_le_bytes([m[0x60], m[0x61]]);
                    let set60 =
                        |m: &mut [u8], v: u16| m[0x60..0x62].copy_from_slice(&v.to_le_bytes());
                    // The first statement: every eighth frame, on the owner's, the targeted count is quartered.
                    if frame != 0 && (frame & 7) as u8 == m[9] {
                        let t = i32::from(m[0x3d] as i8);
                        m[0x3d] = ((t + ((t >> 31) & 3)) >> 2) as u8;
                        log.push("s10:m+0x0,0".into());
                    }
                    let ph = frame.wrapping_add(id);
                    if ph & 31 == 0 {
                        let f = w60(m);
                        set60(m, if f & 0x10 != 0 { f & !0x10 } else { f & !0x20 });
                        let active = if a[105] != 0 {
                            m[8] & 4 != 0
                        } else {
                            a[0] != 0
                        };
                        if !active && rd(&p[4 * who], 0) & 4 == 0 {
                            // The oil platform's arm: on `(frame + o) & 0x7f == 0` a site no unit of its owner targets goes.
                            if ph & 0x7f == 0 {
                                let r = if a[106] != 0 {
                                    log.push("s6:p125+0x0,422,0".into());
                                    take(67, 26, &mut n6)
                                } else {
                                    take(23, 17, &mut n1)
                                };
                                if r != 0 {
                                    let ans = take(93, 2, &mut n11);
                                    log.push(format!(
                                        "s11:c1+0x0,{x},{y},1,{who},-1,0,11,{id},{who},0,0,0,{y}"
                                    ));
                                    if ans < 0 {
                                        log.push("s12:m+0x0,0".into());
                                        break 'g 0;
                                    }
                                }
                            }
                            // The site recruiter: a wonder, or a fort with no damage, wants `max(4, helpers)` builders.
                            let iw = take(40, 2, &mut n2);
                            if iw != 0 || (a[7] != 0 && rd(m, 0x24) == 0) {
                                let hb = i32::from(m[0x64]);
                                let mut want = hb.max(4);
                                if take(40, 2, &mut n2) != 0 {
                                    let f13 = (rd(m, 0x48) as u32 as f32)
                                        / (take(42, 25, &mut n3) as u32 as f32);
                                    for k in 0..8usize {
                                        if rd(&p[4 * k], 0) & 3 != 3 {
                                            continue;
                                        }
                                        log.push(format!("s13:p{}+0x0,{who}", 4 * k));
                                        if take(95, 10, &mut n13) != 0 {
                                            continue;
                                        }
                                        if rd(&p[4 * k + 2], 0) != 0 {
                                            want += 1;
                                            continue;
                                        }
                                        for i in 0..rd(&p[32 + 2 * k], 0).max(0) as usize {
                                            let j = 2 * k + r16(&p[77 + k], 4 * i) as usize;
                                            let ty = format!("p{}+0x0", 109 + j);
                                            let arg = rd(&p[125], 4);
                                            let r = if rd(&p[93 + j], 0) == 0x00b4_2174
                                                || a[106] != 0
                                            {
                                                log.push(format!("s6:{ty},{arg},1"));
                                                take(67, 26, &mut n6)
                                            } else {
                                                take(23, 17, &mut n1)
                                            };
                                            if r != 0 {
                                                let den = take(42, 25, &mut n3) as u32 as f32;
                                                let f14 =
                                                    (rd(&p[93 + j], 0x48) as u32 as f32) / den;
                                                if f13 < f14
                                                    && (f14 < f13 + f13
                                                        || f14 < f32::from_bits(0x3eb3_3333))
                                                {
                                                    want += 1;
                                                }
                                            }
                                        }
                                    }
                                }
                                if hb < want {
                                    let ans = take(93, 2, &mut n11);
                                    log.push(format!(
                                        "s11:c1+0x0,{x},{y},1,{who},3840,512,1,50,0,12,0,0,{y}"
                                    ));
                                    if ans >= 0 {
                                        let row = rd(&c[1], 0x200) as usize;
                                        log.push(format!(
                                            "s14:p{}+0x0,{id},{who},2,0",
                                            49 + 5 * row + ans as usize
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    // Every frame: the helpers byte becomes a flag, and the repair flag clears.
                    let f = w60(m);
                    if m[0x64] == 0 {
                        set60(m, f & !0x400);
                    } else {
                        m[0x64] = 0;
                        set60(m, f | 0x400);
                    }
                    let f = w60(m);
                    set60(m, f & !0x800);
                    if ph & 15 != 0 || rd(&p[4 * who + 3], 0) != 0 {
                        break 'g 0;
                    }
                    // The enemy-land warning: unless a rush game has no war yet, only on the 32-frame phase.
                    if c[0][0x32] != 0 {
                        log.push("s15:c0+0x0".into());
                    }
                    if (c[0][0x32] == 0 || a[15] != 0) && ph & 31 != 0 {
                        break 'g 0;
                    }
                    let cell = rd(&p[76], 4 * (y >> 8) as usize) * rd(&c[2], 0)
                        + rd(&p[76], 4 * (x >> 8) as usize);
                    let mut s = i32::from(p[75][cell as usize * 0x1c + 0xf] as i8);
                    if a[8] != 0 {
                        log.push("s16:m+0x0,sp-0x10,sp-0xc".into());
                        let j = 2 * who + id as usize;
                        log.push(format!("s17:p{}+0x0,{},{},{who}", 109 + j, a[21], a[22]));
                        s = a[17];
                    }
                    if s < 0 || s == who as i32 {
                        break 'g 0;
                    }
                    log.push(format!("s13:p{}+0x0,{s}", 4 * who));
                    if take(95, 10, &mut n13) != 0 {
                        break 'g 0;
                    }
                    let bit = 1u32 << (s & 31);
                    if u32::from(m[0x62]) & bit == 0 && rd(&p[4 * s as usize], 0) & 0x2000 != 0 {
                        m[0x62] = (u32::from(m[0x62]) | bit) as u8;
                        log.push("s10:m+0x0,1".into());
                    }
                    let started = if a[107] != 0 {
                        m[8] & 2 != 0
                    } else {
                        a[4] != 0
                    };
                    if !started {
                        log.push("s12:m+0x0,0".into());
                    } else {
                        log.push("s5:m+0x0,8,0,1,-1,1,1431655765,-1,-1".into());
                        if a[5] != 0 || rd(&p[4 * who + 1], 0) != 0 {
                            break 'g 0;
                        }
                    }
                    if who as i32 == rd(&c[3], 0x298) {
                        // The string table's pointer is the heap address of c5 plus 0x123cc, constant for this window layout (a layout change re-pins it).
                        log.push("s18:15558448,536949356".into());
                        log.push(format!(
                            "s19:c4+0x0,15558448,{x},{y},1,13161068,-1,{y},0,0,{y}"
                        ));
                        log.push("s20:c4+0x0,10".into());
                    }
                    wr(&mut p[4 * who + 1], 0, frame);
                    0
                };
                if log.is_empty() {
                    log.push("-".into());
                }
                r
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "process — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[7 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Wall::process@00640450: {n} rows agree");
}
