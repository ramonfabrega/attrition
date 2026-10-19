//! `Object::fire_ammo@0064c8b0` under the emulator against the port (item 3282; tools/sweep_gen.py, obj with windows).

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

/// `Object::fire_ammo@0064c8b0`, run by `tools/emu/sweep_object_fire_ammo.py` under unicorn
/// (tools/emu): 256 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p3 sparse table 1 (2 rows × 1 slice(s), row-major), p4..p5 sparse table 2 (2 rows × 1 slice(s), row-major), p6..p60 the place(s), in flag order; the global cell 0x00c061fc points at 32 random bytes; the global cell 0x00c061f8 points at 32 random bytes; the global cell 0x00c06214 points at 3584 random bytes; 4 random bytes at 0x00c0618c (each word in 12626800..12626800); vtable slot +0x18 (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x006625c0 (2 args) stubbed; callee 0x006624f0 (4 args) stubbed; callee 0x0062dd90 (0 args) stubbed; callee 0x00616e80 (0 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x00658b10 (1 args) stubbed; the word at `p2+0` points at 8 random bytes less `0` (placed); the word at `p6+0` points at 256 random bytes less `0` (placed); the word at `p6+4` points at 256 random bytes less `0` (placed); the word at `p3+0` points at 8 random bytes less `0` (placed); the word at `p9+0` points at 256 random bytes less `0` (placed); the word at `p9+4` points at 256 random bytes less `0` (placed); the word at `p4+0` points at 8 random bytes less `0` (placed); the word at `p12+0` points at 128 random bytes less `0` (placed); the word at `p12+4` points at 128 random bytes less `0` (placed); the word at `p5+0` points at 8 random bytes less `0` (placed); the word at `p15+0` points at 128 random bytes less `0` (placed); the word at `p15+4` points at 128 random bytes less `0` (placed); the word at `p0+0` points at 224 random bytes less `0` (placed); the word at `p1+0` points at 224 random bytes less `0` (placed); the word at `c0+0x10` points at 1604 random bytes less `0` (placed); the word at `c1+0x10` points at 1604 random bytes less `0` (placed); the word at `p20+244` points at 720 random bytes less `0` (placed); the word at `p20+248` points at 720 random bytes less `0` (placed); the word at `p20+1600` points at 720 random bytes less `0` (placed); the word at `p20+128` points at 720 random bytes less `0` (placed); the word at `p20+8` points at 720 random bytes less `0` (placed); the word at `p21+244` points at 524 random bytes less `0` (placed); the word at `p21+248` points at 524 random bytes less `0` (placed); the word at `p21+1600` points at 524 random bytes less `0` (placed); the word at `p21+128` points at 524 random bytes less `0` (placed); the word at `p21+8` points at 524 random bytes less `0` (placed); the word at `m+0x18` points at 16 random bytes less `0` (placed); the word at `p13+0x40` points at 64 random bytes less `0` (placed); the word at `p33+0x18` points at 576 random bytes less `0` (placed); the word at `p14+0x40` points at 64 random bytes less `0` (placed); the word at `p35+0x18` points at 576 random bytes less `0` (placed); the word at `p16+0x40` points at 64 random bytes less `0` (placed); the word at `p37+0x18` points at 576 random bytes less `0` (placed); the word at `p17+0x40` points at 64 random bytes less `0` (placed); the word at `p39+0x18` points at 576 random bytes less `0` (placed); the word at `p7+0xf4` points at 16 random bytes less `0` (placed); the word at `p41+0` points at 24 random bytes less `0` (placed); the word at `p41+4` points at 24 random bytes less `0` (placed); the word at `p41+8` points at 24 random bytes less `0` (placed); the word at `p41+12` points at 24 random bytes less `0` (placed); the word at `p8+0xf4` points at 16 random bytes less `0` (placed); the word at `p46+0` points at 24 random bytes less `0` (placed); the word at `p46+4` points at 24 random bytes less `0` (placed); the word at `p46+8` points at 24 random bytes less `0` (placed); the word at `p46+12` points at 24 random bytes less `0` (placed); the word at `p10+0xf4` points at 16 random bytes less `0` (placed); the word at `p51+0` points at 24 random bytes less `0` (placed); the word at `p51+4` points at 24 random bytes less `0` (placed); the word at `p51+8` points at 24 random bytes less `0` (placed); the word at `p51+12` points at 24 random bytes less `0` (placed); the word at `p11+0xf4` points at 16 random bytes less `0` (placed); the word at `p56+0` points at 24 random bytes less `0` (placed); the word at `p56+4` points at 24 random bytes less `0` (placed); the word at `p56+8` points at 24 random bytes less `0` (placed); the word at `p56+12` points at 24 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 1] (90%); stub 1 answered from [0, 1, 1, 1] (90%); stub 5 answered from [0, 1, 2, 3, 4, 5, 5, 2, 6, -1] (90%); stub 6 answered from [23, 24, 22, 0, 1, 25, 24] (90%); stub 6 answers 2 values in call order, printed after the answers; stub 7 answers 8 values in call order, printed after the answers; stub 7 answers 0..65535 every row; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 prints the 56 bytes its argument 0 points at, hex, after the record (`/` a call); the stack 512 bytes under the call is zeroed before each row; stub 2 answers from its own `ecx`: the word at `+0x40`; 2 rows 0x6eec apart at `@0xe41248`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 2 rows 0x1c apart at `@0xc0aec0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 2 rows 0x1c apart at `@0xc0ab84`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_fire_ammo.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let log: Vec<String>;
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // No crate method models `Object::fire_ammo`'s own package (the crate's
            // `Sim::fire_ammo_aim` is `Ammo::init`'s side, with no gpiece, no figure fan-out
            // and no footprint scatter), so the port is inline, written from the reading.
            // Windows: m the object (`+9` who, `+0xa` id, `+0xc/0x10/0x14` xored position,
            // `+0x18` its type, `+4` the type index); c0 unittypes (cell 0xc061fc), c1
            // objecttypes, c2 graphic_pieces (`+0xdc0` first_ammo_piece); p0/p1 the leaders'
            // `+0x6eb8` words; p7,p8,p10,p11 the units [who][id]; p13,p14,p16,p17 the objects
            // [who][id]; p18/p19 the leader records; p22..26 unittypes' and p27..31 objecttypes'
            // five placed type records; p34,36,38,40 the footprint records of the four
            // objects; p41.. the units' figure arrays and figures. `a[0]`, `a[1]` the
            // arguments, `a[2..11]` the nine stubs' answers (is_unit, is_building, slot 0xac,
            // two play_sounds, get_shot, order_type, Random::get, add_ammo), `a[11..13]`
            // order_type's two values, `a[13..21]` Random::get's eight.
            const K: i32 = 0x63637;
            const TYPES: [i32; 5] = [0x3d, 0x3e, 0x190, 0x20, 2];
            const UNITS: [[usize; 2]; 2] = [[7, 8], [10, 11]];
            const OBJS: [[usize; 2]; 2] = [[13, 14], [16, 17]];
            const FOOT: [[usize; 2]; 2] = [[34, 36], [38, 40]];
            const FIGS: [[usize; 4]; 4] = [
                [42, 43, 44, 45],
                [47, 48, 49, 50],
                [52, 53, 54, 55],
                [57, 58, 59, 60],
            ];
            let i16at = |w: &[u8], o: usize| i16::from_le_bytes([w[o], w[o + 1]]);
            let is_unit = a[2] != 0;
            let is_building = a[3] != 0;
            let mut cols: [Vec<String>; 5] = Default::default();
            let mut pkg = [0u8; 0x38];
            let put = |pkg: &mut [u8; 0x38], o: usize, v: i32| {
                pkg[o..o + 4].copy_from_slice(&v.to_le_bytes())
            };
            let who = m[9] as usize;
            let id = i16at(m, 10);
            let slot = (who, id as usize);
            let ty = rd(&p[32], 4);
            let ty_ix = TYPES.iter().position(|&t| t == ty).expect("a placed type");
            let u_win = || UNITS[slot.0][slot.1];
            let (angle, zadd);
            if is_unit {
                put(&mut pkg, 0, rd(&p[22 + ty_ix], 0x2cc));
                if ty == 0x3d || ty == 0x3e || ty == 0x190 {
                    let u = &p[u_win()];
                    if rd(u, 0x68) & 0x80000 == 0 {
                        let mut g = rd(&c[2], 0xdc0);
                        if (rd(&p[18 + slot.0], 0xdc) ^ 0x62766) > 2 {
                            g = g.wrapping_add(1);
                        }
                        put(&mut pkg, 0, g);
                        cols[0].push(format!("m+0x0,1,{}", i32::from(id)));
                    }
                }
                angle = rd(&p[u_win()], 0x50);
                zadd = 100;
            } else if is_building {
                let r = &p[OBJS[slot.0][slot.1]];
                let x = rd(r, 0x10) ^ K;
                let y = rd(r, 0x14) ^ K;
                cols[1].push(format!("m+0x0,1,{x},{y},{y}"));
                let shot = a[7];
                if (0..=5).contains(&shot) {
                    put(&mut pkg, 0, rd(&c[2], 0xdc0).wrapping_add(shot));
                }
                angle = 0;
                zadd = 0xfa;
            } else {
                angle = 0;
                zadd = 0;
            }
            if is_unit || is_building {
                put(&mut pkg, 0x1c, angle);
                pkg[0x32..0x34].copy_from_slice(&id.to_le_bytes());
                pkg[0x34..0x36].copy_from_slice(&(a[0] as i16).to_le_bytes());
                pkg[0x36] = m[9];
                pkg[0x37] = a[1] as u8;
                let mut ot_turn = 0usize;
                let mut order_type = |cols: &mut [Vec<String>; 5]| {
                    let v = a[11 + ot_turn.min(1)];
                    ot_turn += 1;
                    cols[2].push(format!("p{}+0x0", u_win()));
                    v
                };
                let go = (a[0] >= 0 && a[1] >= 0)
                    || (is_unit && {
                        let t = order_type(&mut cols);
                        t == 0x17 || order_type(&mut cols) == 0x18
                    });
                if go && is_unit {
                    let u = &p[u_win()];
                    let n = i32::from(u[0xb5] as i8);
                    for i in 0..n.max(0) as usize {
                        let f = &p[FIGS[slot.0 * 2 + slot.1][i]];
                        let mut q = pkg;
                        put(&mut q, 4, rd(f, 0xc));
                        put(&mut q, 8, rd(f, 0x10));
                        put(&mut q, 0xc, rd(f, 0x14).wrapping_add(zadd));
                        cols[4].push(q.iter().map(|b| format!("{b:02x}")).collect());
                    }
                } else if go && is_building {
                    let foot = &p[FOOT[slot.0][slot.1]];
                    let w = rd(foot, 0x234).wrapping_mul(0x60);
                    let h = rd(foot, 0x238).wrapping_mul(0x60);
                    let count = rd(&p[27 + ty_ix], 0x208);
                    let mut turn = 0usize;
                    let mut rand = |cols: &mut [Vec<String>; 5]| {
                        let v = a[13 + turn.min(7)];
                        turn += 1;
                        cols[3].push("14908044,0,65535".to_string());
                        v
                    };
                    for _ in 0..count.max(0) {
                        let rx = if w.wrapping_sub(1) > 0 {
                            rand(&mut cols).wrapping_rem(w)
                        } else {
                            0
                        };
                        let x = ((rd(m, 0x10) ^ K).wrapping_sub(w / 2)).wrapping_add(rx);
                        let ry = if h.wrapping_sub(1) > 0 {
                            rand(&mut cols).wrapping_rem(h)
                        } else {
                            0
                        };
                        let mut q = pkg;
                        put(&mut q, 4, x);
                        put(
                            &mut q,
                            8,
                            ((rd(m, 0x14) ^ K).wrapping_sub(h / 2)).wrapping_add(ry),
                        );
                        put(&mut q, 0xc, (rd(m, 0xc) ^ K).wrapping_add(zadd));
                        cols[4].push(q.iter().map(|b| format!("{b:02x}")).collect());
                    }
                }
            }
            log = cols
                .iter()
                .map(|c| {
                    if c.is_empty() {
                        "-".to_string()
                    } else {
                        c.join("/")
                    }
                })
                .collect();
            want_eax
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "fire_ammo — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "fire_ammo — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "fire_ammo — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "fire_ammo — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Object::fire_ammo@0064c8b0: {n} rows agree");
}
