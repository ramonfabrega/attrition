//! The occupied ammo pool in slot order. Each ordinary sprite contributes
//! its flags, its 99-byte display record and the absent-path byte.
use super::{Line, Range};
use crate::Sim;

impl Sim {
    pub fn checksum_ammo(&self) -> Option<Line> {
        let mut ranges = Vec::with_capacity(self.combat.projectiles.len() * 3);
        for p in &self.combat.projectiles {
            // Guided paths and falling aircraft have separate display records.
            if p.graphic.guided || p.spline.is_some() || p.crash.is_some() {
                return None;
            }
            let (o, _) = self.obj_o_who(p.shooter);
            let (ox, whom) = p.target.map_or((-1, -1), |t| self.obj_o_who(t));
            let flags = if p.spent {
                1
            } else {
                2 | (u8::from(p.rolling) << 2)
                    | (u8::from(p.missed) << 3)
                    | (u8::from(p.harmless) << 4)
            };
            let mut body = Vec::with_capacity(99);
            body.push(0); // rolling display byte (separate from flag 4)
            body.extend((p.accuracy as i16).to_le_bytes());
            for word in [
                p.gpiece,
                p.launch.x,
                p.launch.y,
                p.sz,
                p.landing.x,
                p.landing.y,
                p.ez,
                p.cur_time,
                p.total_time,
                p.angle.0,
                p.splash_area,
                p.slot as i32,
                p.graphic.graph_index as i32,
                i32::from(p.owner),
                o as i32,
                p.num_guys,
                whom as i32,
                ox as i32,
                1,
                p.v1z.bits() as i32,
                p.graphic.dx.bits() as i32,
                0,
                0,
                0, // bank_dx, bank_dy, start_roll_angle
            ] {
                body.extend(word.to_le_bytes());
            }
            for (field, bytes) in [
                ("flags", vec![flags]),
                ("rolling..start_roll_angle", body),
                ("has_path", vec![0]),
            ] {
                ranges.push(Range {
                    record: p.slot as i16,
                    field,
                    bytes,
                });
            }
        }
        Some(Line {
            name: "ammo",
            ranges,
        })
    }
}
