//! `Object::update_seen@00651b80` under the emulator against the port (item 3268; tools/sweep_gen.py, obj with windows).

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

/// `Object::update_seen@00651b80`, run by `tools/emu/sweep_object_update_seen.py` under unicorn
/// (tools/emu): 128 random bytes of object, 1 argument(s), `p` windows in order: p0..p4 the place(s), in flag order, p5 stub 2's answer window; the global cell 0x00cae5fc points at 2048 random bytes; the global cell 0x00c06188 points at 32 random bytes; the global cell 0x00c0618c points at 80 random bytes; 56 random bytes at 0x00c0aec0; vtable slot +0x2c (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x128 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x164 (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; callee 0x006b3c60 (4 args) stubbed; callee 0x006b3d30 (4 args) stubbed; callee 0x006b4bb0 (4 args) stubbed; callee 0x0055e0ac (3 args) stubbed; the word at `m+0x18` points at 768 random bytes less `0` (placed); the word at `c2+0x14` points at 8 random bytes less `0` (placed); the word at `c2+0x30` points at 8 random bytes less `0` (placed); the word at `c3+0x0` points at 8 random bytes less `0` (placed); the word at `c3+0x1c` points at 8 random bytes less `0` (placed); stub 2 answers the address of 32 random bytes, a `p` window after the places; the word at `p5+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p1+0` laid as the address of `m` every row; the word at `p1+4` laid as the address of `m` every row; the word at `p2+0` laid as the address of `m` every row; the word at `p2+4` laid as the address of `m` every row; the word at `p3+0` laid as the address of `m` every row; the word at `p3+4` laid as the address of `m` every row; the word at `p4+0` laid as the address of `m` every row; the word at `p4+4` laid as the address of `m` every row; stub 3 answers 0..70 every row; stub 4 answers 1..1 every row; stub 6 answers 0..0 every row; stub 7 answers 1..1 every row; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 7 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 8 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 8 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 9 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 9 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 10 is `__cdecl`: the caller pops its arguments; the stack 256 bytes under the call is zeroed before each row; `0x006817f0` run once before the rows (a table builder the startup would have run); `0x00681920` run once before the rows (a table builder the startup would have run); stub 10 copies its argument 2 bytes from argument 1 to argument 0 (a `memcpy`); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 300 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_update_seen.py") else {
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
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            port(a, m, c, p, &mut log)
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "update_seen — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_seen — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "update_seen — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "update_seen — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("Object::update_seen@00651b80: {n} rows agree");
}

/// The port: `Sim::update_seen` for a unit, on a `Sim` laid from the row's windows (item 3268).
///
/// `a[0]` is `param_1` (the ring pass), `a[1 + k]` stub `k`'s answer: `a[4]` the `+0x128` slot
/// (`mylos`), `a[5]` the `+0x18` slot (`is_unit`, held to 1: the building arm is
/// `Sim::update_seen_build`'s, a different function). Windows: `m` the object (`+0x8` flags,
/// `+0x9` owner, `+0xa` index, `+0x10`/`+0x14` the position xor `0x63637`, `+0x40` `visible`, `+0x50`
/// `UnitData::angle`), `c[1]` the World cell (`+0xc`/`+0x10` the fog grid), `p[0]` the unit's data
/// (`+0x218` the type's domain, `+0x2b8` its `unit_flags2`). The circle and ring tables are the
/// original's own (`circle_init@006817f0`, `ring_init@00681920` and `trig_init@00a46980` — the sine table `project` reads — run before the rows) against this
/// crate's `circle()`/`ring()`. The recorded calls the original makes are derived from the crate's
/// own `Sim::seen_sweep`, then `Sim::update_seen` is run and its planes and count held to them.
pub(super) fn port(
    a: &[i32],
    m: &[u8],
    c: &[Vec<u8>],
    p: &[Vec<u8>],
    log: &mut Vec<String>,
) -> i64 {
    use crate::ai_load::uflags2;
    use crate::ai_place::circle;
    use crate::attrition::Domain;
    use crate::combat::mask;
    use crate::movement::Angle;
    use crate::vision::ring;
    use crate::world::{Cell, Pos, Terrain, World};
    assert_eq!(a[5], 1, "the sweep holds is_unit to 1");
    let (ring_pass, mylos) = (a[0] != 0, a[4]);
    let (flags, owner, visible) = (m[8], m[9], m[0x40]);
    let idx = i32::from(i16::from_le_bytes([m[0xa], m[0xb]]));
    let (px, py) = (rd(m, 0x10) ^ 0x63637, rd(m, 0x14) ^ 0x63637);
    let (fw, fh) = (rd(&c[1], 0xc), rd(&c[1], 0x10));
    let d = &p[0];
    let sea = rd(d, 0x218) != 0;
    let packs = d[0x2b8] & 4 != 0;
    let detect = flags & 0x40 != 0;
    let heading = rd(m, 0x50);

    let mut world = World::new(fw / 2, fh / 2);
    world.fill_region(
        Terrain::Land,
        Cell::new(0, 0),
        Cell::new(fw / 2 - 1, fh / 2 - 1),
    );
    assert!(world.set_fog(vec![0; (fw * fh) as usize]));
    let mut s = crate::Sim::new(crate::Tuning::RON, world, 2);
    let mut ty = crate::UnitType {
        hits: 20,
        ..crate::UnitType::default()
    };
    ty.kind.domain = if sea { Domain::Sea } else { Domain::Land };
    if packs {
        ty.cols.unit_flags2 |= uflags2::PACKS;
    }
    if detect {
        ty.combat.obj_masks |= mask::DETECT;
    }
    let t = s.add_unit_type(ty);
    let mut unit = crate::Unit::new(owner, 0, Pos::new(px, py), 20);
    unit.ty = Some(t);
    unit.mylos = mylos;
    unit.visible = visible;
    unit.movement.heading = Angle(heading);
    let u = s.add_unit(unit);
    // Placing the unit derived its own `mylos` and may have lit its own disc: the row's values are
    // laid back, and it starts in the dark.
    s.units[u].mylos = mylos;
    s.units[u].visible = visible;
    s.units[u].pos = Pos::new(px, py);
    s.units[u].movement.heading = Angle(heading);
    assert!(s.world.set_fog(vec![0; (fw * fh) as usize]));
    s.world.clear_seen();

    let bit = 1u8 << owner;
    let mut cells: Vec<(i32, i32)> = Vec::new();
    let sweep = s.seen_sweep(u, ring_pass);
    if let Some(sw) = &sweep {
        let (xs, ys): (&[i32], &[i32]) = if sw.ring {
            let g = ring();
            (&g.x, &g.y)
        } else {
            let g = circle();
            (&g.x, &g.y)
        };
        for i in sw.start..sw.end {
            let (x, y) = (xs[i] + sw.centre.0, ys[i] + sw.centre.1);
            if x >= 0 && y >= 0 && x < fw && y < fh {
                cells.push((x, y));
            }
        }
    }
    // The original's quirk (`651d91`–`651dac`): a detector's ring pass at radius 32 clamps `r` to
    // `0x1f` in place, and so walks the empty slice `[ring_radius[31], ring_radius[31])`. The first run
    // of this sweep parted here (the crate lit the last ring); `Sim::update_seen` now returns early.
    if detect && sweep.is_some_and(|sw| sw.ring && sw.radius >= 0x20) {
        cells.clear();
    }
    // `vtable +0x164` first: whole-disc call, `visible` set (the unit's local_18 is held to 0).
    let local = sweep.is_some() && !ring_pass && visible != 0;
    let le = |v: i32| {
        v.to_le_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    };
    let join = |v: Vec<String>| {
        if v.is_empty() {
            "-".to_string()
        } else {
            v.join("/")
        }
    };
    let det = i32::from(detect);
    log.push(if local { "m+0x0".into() } else { "-".into() });
    log.push(join(
        cells
            .iter()
            .map(|&(_, y)| format!("{y},sp-0x20,sp-0x14,{owner},{det}"))
            .collect(),
    ));
    log.push(join(
        cells
            .iter()
            .map(|&(_, y)| format!("{y},sp-0x20,sp-0x14,{owner},{idx}"))
            .collect(),
    ));
    log.push("-".into());
    log.push(join(cells.iter().map(|&(x, _)| le(x)).collect()));
    log.push(join(cells.iter().map(|&(_, y)| le(y)).collect()));
    log.push(join(cells.iter().map(|&(x, _)| le(x)).collect()));
    log.push(join(cells.iter().map(|&(_, y)| le(y)).collect()));
    log.push("-".into());
    log.push("-".into());

    // Now the crate's own pass, and its planes held to the same cells.
    let got = s.update_seen(u, ring_pass);
    let lit = (
        s.units[u].pos.x.div_euclid(crate::vision::UNITS_PER_FOG),
        s.units[u].pos.y.div_euclid(crate::vision::UNITS_PER_FOG),
    );
    // `update_local_seen_unit` lights the unit's own half-cell with `visible` first, which
    // `World::set_seen` then reports unchanged at that cell when the owner's bit is in it.
    // The ring table repeats a point or two (its gap patches), and a repeat is not newly seen the
    // second time: the stub answers 1 to every call, the real `set_seen` would not.
    let mut uniq = cells.clone();
    uniq.sort_unstable();
    uniq.dedup();
    let mut want_new = uniq.len();
    let mut seen = vec![0u8; (fw * fh) as usize];
    let mut seen3 = vec![0u8; (fw * fh) as usize];
    let at = |x: i32, y: i32| (y * fw + x) as usize;
    if local && lit.0 >= 0 && lit.1 >= 0 && lit.0 < fw && lit.1 < fh {
        seen[at(lit.0, lit.1)] |= visible;
        if cells.contains(&lit) && visible & bit != 0 {
            want_new -= 1;
        }
    }
    for &(x, y) in &cells {
        seen[at(x, y)] |= bit;
        if detect {
            seen3[at(x, y)] |= bit;
        }
    }
    assert_eq!(got, want_new, "update_seen — the count of cells newly seen");
    for y in 0..fh {
        for x in 0..fw {
            assert_eq!(
                s.world.seen(x, y),
                Some(seen[at(x, y)]),
                "update_seen — seen at ({x}, {y})"
            );
            assert_eq!(
                s.world.seen2(x, y),
                Some(seen[at(x, y)]),
                "update_seen — seen2 at ({x}, {y})"
            );
            assert_eq!(
                s.world.seen3(x, y),
                seen3[at(x, y)],
                "update_seen — seen3 at ({x}, {y})"
            );
        }
    }
    0
}

/// The port for the building arm (`is_unit` held to 0, `param_1` to 0): `Sim::update_seen_build`, on
/// a `Sim` laid from the row (item 3268, `object_update_seen_build`). The disc is `Sim::build_sweep`'s;
/// the `+0x164` call's gate is the original's `visible || local_18`, where `local_18` is
/// `is_wonder() (slot +0x2c) && !(flags & 0x20) && !(slot +0xfc) && started` — the crate's
/// `visible != 0 || (started && is_visible_wonder)`, the city flag and the fort test laid as the
/// type's lineage. `started` is held true (slot `+0x50` answers 1, `Y+8 & 2` is set): the crate's
/// `build_los` is 0 for an unstarted building, which the original's `mylos` stub does not tie.
pub(super) fn port_build(
    a: &[i32],
    m: &[u8],
    c: &[Vec<u8>],
    _p: &[Vec<u8>],
    log: &mut Vec<String>,
) -> i64 {
    use crate::build::{BuildType, Ident};
    use crate::world::{Cell, Pos, Terrain, World};
    assert_eq!(
        (a[0], a[5]),
        (0, 0),
        "the sweep holds ring to 0 and is_unit to 0"
    );
    assert_eq!(a[7], 1, "slot +0x50 (started) answers 1");
    let mylos = a[4];
    let (wonder, fort) = (a[1] != 0, a[2] != 0);
    let (flags, owner, visible) = (m[8], m[9], m[0x40]);
    let idx = i32::from(i16::from_le_bytes([m[0xa], m[0xb]]));
    let (px, py) = (rd(m, 0x10) ^ 0x63637, rd(m, 0x14) ^ 0x63637);
    let (fw, fh) = (rd(&c[1], 0xc), rd(&c[1], 0x10));
    let city = flags & 0x20 != 0;
    assert_eq!(
        flags & 0x40,
        0,
        "no building carries DETECT: the crate lights no seen3 for one"
    );
    let x_size = mylos.rem_euclid(5);
    let mk = || {
        let mut world = World::new(fw / 2, fh / 2);
        world.fill_region(
            Terrain::Land,
            Cell::new(0, 0),
            Cell::new(fw / 2 - 1, fh / 2 - 1),
        );
        assert!(world.set_fog(vec![0; (fw * fh) as usize]));
        let mut s = crate::Sim::new(crate::Tuning::RON, world, 2);
        let fort_t = s.add_build_type(BuildType {
            ident: Ident::Fort,
            ..BuildType::default()
        });
        let ident = if city {
            Ident::Village
        } else if fort {
            Ident::Fort
        } else {
            BuildType::default().ident
        };
        let t = s.add_build_type(BuildType {
            ident,
            from: (city && fort).then_some(fort_t),
            x_size,
            y_size: x_size,
            los: mylos - x_size / 2,
            wonder,
            ..BuildType::default()
        });
        let b = s.add_building(owner, Pos::new(px, py), 0);
        s.buildings[b].ty = Some(t);
        s.buildings[b].alive = true;
        s.buildings[b].started = true;
        s.buildings[b].active = true;
        s.buildings[b].visible = visible;
        (s, b)
    };
    let (mut s, b) = mk();
    let sweep = s.build_sweep(b);
    let bit = 1u8 << owner;
    let mut cells: Vec<(i32, i32)> = Vec::new();
    if let Some(sw) = &sweep {
        let g = crate::ai_place::circle();
        for i in sw.start..sw.end {
            let (x, y) = (g.x[i] + sw.centre.0, g.y[i] + sw.centre.1);
            if x >= 0 && y >= 0 && x < fw && y < fh {
                cells.push((x, y));
            }
        }
    }
    let local_18 = wonder && !city && !fort;
    let local = sweep.is_some() && (visible != 0 || local_18);
    let le = |v: i32| {
        v.to_le_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    };
    let join = |v: Vec<String>| {
        if v.is_empty() {
            "-".to_string()
        } else {
            v.join("/")
        }
    };
    log.push(if local { "m+0x0".into() } else { "-".into() });
    log.push(join(
        cells
            .iter()
            .map(|&(_, y)| format!("{y},sp-0x20,sp-0x14,{owner},0"))
            .collect(),
    ));
    log.push(join(
        cells
            .iter()
            .map(|&(_, y)| format!("{y},sp-0x20,sp-0x14,{owner},{idx}"))
            .collect(),
    ));
    log.push("-".into());
    log.push(join(cells.iter().map(|&(x, _)| le(x)).collect()));
    log.push(join(cells.iter().map(|&(_, y)| le(y)).collect()));
    log.push(join(cells.iter().map(|&(x, _)| le(x)).collect()));
    log.push(join(cells.iter().map(|&(_, y)| le(y)).collect()));
    log.push("-".into());
    log.push("-".into());

    // The reference: the `+0x164` write alone, then the disc's cells on top; the crate's own pass
    // (one call) is held to its planes and its count.
    let (mut r, rb) = mk();
    if local {
        r.update_local_seen_build(rb);
    }
    let at = |x: i32, y: i32| (y * fw + x) as usize;
    let mut uniq = cells.clone();
    uniq.sort_unstable();
    uniq.dedup();
    let want_new = uniq
        .iter()
        .filter(|&&(x, y)| r.world.seen2(x, y).is_some_and(|v| v & bit == 0))
        .count();
    let mut seen = vec![0u8; (fw * fh) as usize];
    let mut seen2 = vec![0u8; (fw * fh) as usize];
    for y in 0..fh {
        for x in 0..fw {
            seen[at(x, y)] = r.world.seen(x, y).unwrap_or(0);
            seen2[at(x, y)] = r.world.seen2(x, y).unwrap_or(0);
        }
    }
    for &(x, y) in &cells {
        seen[at(x, y)] |= bit;
        seen2[at(x, y)] |= bit;
    }
    let got = s.update_seen_build(b);
    assert_eq!(
        got, want_new,
        "update_seen_build — the count of cells newly seen"
    );
    for y in 0..fh {
        for x in 0..fw {
            assert_eq!(
                s.world.seen(x, y),
                Some(seen[at(x, y)]),
                "update_seen_build — seen at ({x}, {y})"
            );
            assert_eq!(
                s.world.seen2(x, y),
                Some(seen2[at(x, y)]),
                "update_seen_build — seen2 at ({x}, {y})"
            );
            assert_eq!(
                s.world.seen3(x, y),
                0,
                "update_seen_build — seen3 at ({x}, {y})"
            );
        }
    }
    0
}
