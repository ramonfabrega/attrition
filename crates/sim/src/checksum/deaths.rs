//! The occupied death pool, in the slot order of `check_deaths` (CHECKSUM §8).
use super::{Line, Range};
use crate::Sim;
use crate::units::anim::SQUAD_SIZE;

impl Sim {
    /// Fold each live figure's valid word and its 71-byte body. A hit whose
    /// ammo arguments have not been retained leaves the entire line unknown.
    pub fn checksum_deaths(&self) -> Option<Line> {
        let mut ranges = Vec::new();
        for (slot, entry) in self.combat.death_pool.iter().enumerate() {
            let Some(entry) = entry else { continue };
            // Free slots can still point at a culled life or an expired crew
            // member of a unit whose other figure is still present.
            let Some(d) = self.live_death(entry) else {
                continue;
            };
            let k = usize::from(entry.figure);
            let age = self.frame - d.first_frame;
            let (ammo, angle) = d.impact?;
            let piece = d.figures.gpiece[k];
            let place = d.figures.place[k];
            let skeleton = self
                .roster
                .art
                .piece_skeletons
                .get(&piece)
                .copied()
                .unwrap_or(-1);
            let words = [
                d.first_frame as i32,
                d.anim_of(k),
                place.pos.x,
                place.pos.y,
                place.z,
                d.who,
                d.o,
                piece,
                ammo,
                angle.bits() as i32,
                place.angle.0,
                place.turret_angles[0],
                place.turret_angles[1],
                place.turret_angles[2],
                place.turret_angles[3],
                age as i32,
                skeleton,
            ];
            let mut bytes = Vec::with_capacity(71);
            for word in words {
                bytes.extend(word.to_le_bytes());
            }
            bytes.extend((-1i16).to_le_bytes()); // node_flags at init
            bytes.push(k.checked_sub(SQUAD_SIZE).map_or(0xff, |crew| crew as u8));
            let record = slot as i16;
            ranges.push(Range {
                record,
                field: "valid",
                bytes: 1i32.to_le_bytes().to_vec(),
            });
            ranges.push(Range {
                record,
                field: "first_frame..unit_crew",
                bytes,
            });
        }
        if ranges.is_empty() && !self.combat.deaths.is_empty() {
            return None;
        }
        Some(Line {
            name: "deaths",
            ranges,
        })
    }
}
