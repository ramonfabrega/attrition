//! The surf (item 4236; `docs/RENDERER.md` §3.26).
//!
//! The waves and breakers `Surf::init@008a0cf0` lays along the shore at map
//! load, each drawn by `SurfWaveSection::draw@0089e360` from its section
//! models (`render::mot`) at its timing's frame. Their phase and their
//! shape are draws on **`internal_random`** (`0x00eb697c`), the renderer's
//! own generator, which the simulation never steps.

/// `internal_random`'s draws before `Surf::init`'s first, on run1567's
/// trace (the GreatLakes lobby's map, item 4236): 45 under
/// `RiverDoobers::init_class`, 58,081 in `Terrain::init`'s
/// `random_map_pool` (`(4 · width + 1)²`, width 60), 45 under
/// `RiverDoobers::init_class` again, 360 in `TerrainOil::add_rocky_patch`
/// and 713 in `RiverSectionOut::create_doobers`.
pub const DRAWS_BEFORE_SURF_1567: u32 = 59_244;

/// `internal_random`: `Random`'s generator (`Random::get@00a39d70`), a
/// 32-bit LCG that starts at 0 (`Random::Random@00a39d50`) and that no
/// site reseeds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InternalRandom(pub u32);

impl InternalRandom {
    /// One step.
    pub fn step(&mut self) {
        self.0 = self.0.wrapping_mul(0x0019_660d).wrapping_add(0x3c6e_f35f);
    }

    /// The generator after `n` draws from 0.
    pub fn after(n: u32) -> Self {
        let mut r = Self(0);
        for _ in 0..n {
            r.step();
        }
        r
    }

    /// `Random::get(lo, hi)`: `lo` when the two are equal, without a step;
    /// otherwise a step, and the low 16 bits scaled onto `[lo, hi)`.
    pub fn get(&mut self, lo: i32, hi: i32) -> i32 {
        if lo == hi {
            return lo;
        }
        let (lo, hi) = (lo.min(hi), lo.max(hi));
        self.step();
        ((i64::from(self.0 & 0xffff) * i64::from(hi - lo)) >> 16) as i32 + lo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    /// `internal_random` in VA.
    const INTERNAL_RANDOM: u32 = 0x00eb_697c;

    fn index() -> Vec<(u32, String)> {
        let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
        let Ok(text) = std::fs::read_to_string(home.join("ghidra-projects/decomp/INDEX.tsv"))
        else {
            return vec![];
        };
        let mut v: Vec<(u32, String)> = text
            .lines()
            .filter_map(|l| {
                let mut f = l.split('\t');
                let a = u32::from_str_radix(f.next()?, 16).ok()?;
                Some((a, f.next()?.to_string()))
            })
            .collect();
        v.sort();
        v
    }

    fn function(index: &[(u32, String)], va: u32) -> String {
        let i = index.partition_point(|e| e.0 <= va);
        i.checked_sub(1)
            .map_or_else(|| format!("{va:x}"), |i| index[i].1.clone())
    }

    /// **`internal_random` at `Surf::init`** (item 4236; `docs/RENDERER.md`
    /// §3.26): on run1567's trace (`rontrace.log`, every `Random::get` with
    /// its `Random *` and the word before the step), the generator's first
    /// draw finds it at 0, every draw follows the last by one step, and the
    /// draws before `Surf::init`'s first are 59,244, all from map-load
    /// sites, in this order. So the state `Surf::init` starts from is the
    /// map's, not the session's: nothing in the menu or the lobby draws
    /// it. After setup it is drawn each frame by `Surf::inc_time`, the
    /// river wakes, the ocean floor's clock, the particles and the ammo.
    #[test]
    fn internal_random_at_surf_init_is_the_map_s() {
        let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
        let path = home.join("ron-golden/run1567/map-14/rontrace.log");
        let Ok(Some(trace)) = rondata::trace::Trace::read(&path) else {
            println!("skipping: no trace at {}", path.display());
            return;
        };
        let index = index();
        if index.is_empty() {
            println!("skipping: no decompile index");
            return;
        }
        let ours: Vec<_> = trace
            .draws()
            .iter()
            .filter(|d| d.rng == INTERNAL_RANDOM && matches!(d.kind, 1 | 3))
            .collect();
        assert_eq!(ours[0].seed, 0, "the first draw finds the generator at 0");
        let mut r = InternalRandom(0);
        for d in &ours {
            assert_eq!(d.seed, r.0, "a step between two draws");
            r.step();
        }
        let first = ours
            .iter()
            .position(|d| function(&index, d.site).starts_with("Surf::"))
            .unwrap();
        assert_eq!(first as u32, DRAWS_BEFORE_SURF_1567);
        assert_eq!(
            ours[first].seed,
            InternalRandom::after(DRAWS_BEFORE_SURF_1567).0
        );
        let mut runs: Vec<(String, u32)> = vec![];
        for d in &ours[..first] {
            let caller = format!(
                "{} < {}",
                function(&index, d.site),
                function(&index, d.up[0])
            );
            match runs.last_mut() {
                Some(r) if r.0 == caller => r.1 += 1,
                _ => runs.push((caller, 1)),
            }
        }
        assert_eq!(
            runs,
            [
                ("RiverDooberType::init < RiverDoobers::init_class", 45),
                ("Terrain::init < Setup::build_game", 58_081),
                ("RiverDooberType::init < RiverDoobers::init_class", 45),
                (
                    "TerrainOil::add_rocky_patch < TerrainOil::create_doobers",
                    360
                ),
                ("RiverSectionOut::create_doobers < RiverSection::init", 713),
            ]
            .map(|(s, n)| (s.to_string(), n))
        );
        let mut after: BTreeMap<String, u32> = BTreeMap::new();
        for d in ours.iter().filter(|d| d.frame >= 0) {
            let f = function(&index, d.site);
            *after.entry(f).or_default() += 1;
        }
        let names: Vec<&str> = after.keys().map(String::as_str).collect();
        assert_eq!(
            names,
            [
                "AmmoOut::dirt_splash",
                "AmmoOut::explode",
                "Particle::init",
                "RiverSectionOut::update_wake_polys",
                "Surf::inc_time",
                "TerrainOut::modify_ocean_floor_texture",
            ],
            "the generator's consumers after setup, frames 0 to 3293"
        );
    }
}
