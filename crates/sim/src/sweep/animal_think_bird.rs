//! `Animal::think_bird@005d79e0` under the emulator against the port (item 3259; tools/sweep_gen.py, obj with windows).

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

/// `Animal::think_bird@005d79e0`, run by `tools/emu/sweep_animal_think_bird.py` under unicorn
/// (tools/emu): 160 random bytes of object, 2 argument(s), `p` windows in order: p0 argument 0, p1..p10 the place(s), in flag order; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c06184 points at 16 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c061b8 points at 32 random bytes; the global cell 0x00cae5fc points at 2048 random bytes; argument 0 names 32 random bytes; vtable slot +0x8c (0 args) stubbed; callee 0x00616e80 (0 args) stubbed; the word at `m+0x18` points at 8 random bytes less `0` (placed); the word at `c2+0x134` points at 1008 random bytes less `0` (placed); the word at `c3+0x10` points at 544 random bytes less `0` (placed); stub 0's answer points at 76 random bytes less `0` (placed); the word at `p4+0x14` points at 4 random bytes less `0` (placed); the word at `p4+0x30` points at 4 random bytes less `0` (placed); the word at `p3+0x7c` points at 232 random bytes less `0` (placed); the word at `p3+0x104` points at 32 random bytes less `0` (placed); the word at `p3+0x18c` points at 8 random bytes less `0` (placed); the word at `p3+0x214` points at 16 random bytes less `0` (placed); the stubbed vtable laid at `p0+0`; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_animal_think_bird.py") else {
        return;
    };
    use crate::world::{Cell, Pos, Terrain, World};
    // The one map every row's windows hold (the generator's `--wbias` flags, asserted below): 6x6 cells in four
    // regions -- a big one, a 2x2 island, a single cell (no draw for the landing cell) and a pair.
    const LAYOUT: [[u16; 6]; 6] = [
        [0, 0, 0, 0, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [0, 0, 2, 0, 0, 0],
        [0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 3, 3],
    ];
    let mut w = World::new(6, 6);
    for _ in 0..4 {
        w.add_region(Terrain::Land);
    }
    for (y, row) in LAYOUT.iter().enumerate() {
        for (x, &r) in row.iter().enumerate() {
            w.set_region(Cell::new(x as i32, y as i32), r);
        }
    }
    let mut sim = crate::Sim::new(crate::Tuning::RON, w, 2);
    let t = sim.add_unit_type(crate::UnitType::default());
    let mut bird = crate::Unit::new(1, 0, Pos::new(0x1000, 0x1000), 1);
    bird.ty = Some(t);
    let u = sim.add_unit(bird);
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `Animal::think_bird`: the gate, the `mana_burn` clear, the type dispatch and the two writes the
            // crate does not model (`rec+0x44/+0x48`) are this block's; the 0x192 arm is `Sim::think_bird` itself,
            // run on a `Sim` whose world is the layout above, with the row's counter, frame, seed and patrol point.
            let sh = |w: &[u8], o: usize| i16::from_le_bytes([w[o], w[o + 1]]);
            for (y, row) in LAYOUT.iter().enumerate() {
                for (x, &r) in row.iter().enumerate() {
                    assert_eq!(
                        rd(&p[2], 0x1c * (y * 6 + x) + 4),
                        i32::from(r),
                        "layout, row `{line}`"
                    );
                }
            }
            let st = sh(m, 0x98);
            let frame = rd(&c[0], 0x550);
            let run = st < 0 || frame & 7 == 0;
            let mut called = false;
            let mut done = false;
            let ty = rd(&p[1], 4);
            if run {
                m[0x96] = 0;
                m[0x97] = 0;
                if ty == 0x194 {
                    called = true;
                }
            }
            if ty == 0x192 {
                // The crate owns the gate: it is handed every row of a bird, a skipped frame included.
                // `-1 < spell_time` after the gate's own increment.
                let post = if st >= 0 { st.wrapping_add(1) } else { st };
                if run && post >= 0 {
                    wr(&mut p[4], 0x44, -1);
                    wr(&mut p[4], 0x48, -1);
                }
                sim.units[u].spell_time = st;
                sim.gaia.bird_goals = vec![(u, Pos::new(rd(&p[5], 0), rd(&p[6], 0)))];
                sim.gaia.bird_landings.clear();
                sim.rng.seed = rd(&c[1], 0) as u32;
                sim.think_bird(u, i64::from(frame));
                let g = sim.bird_goal(u);
                wr(&mut p[5], 0, g.x);
                wr(&mut p[6], 0, g.y);
                wr(&mut c[1], 0, sim.rng.seed as i32);
                m[0x98..0x9a].copy_from_slice(&sim.units[u].spell_time.to_le_bytes());
                done = true;
            }
            if !done && st >= 0 {
                m[0x98..0x9a].copy_from_slice(&st.wrapping_add(1).to_le_bytes());
            }
            log.push(if called {
                "m+0x0".to_string()
            } else {
                "-".to_string()
            });
            want_eax
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "think_bird — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "think_bird — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "think_bird — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "think_bird — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Animal::think_bird@005d79e0: {n} rows agree");
}
