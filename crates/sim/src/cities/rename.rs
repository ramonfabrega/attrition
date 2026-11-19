//! **A city's name, as a person writes it** — `CommandPackage::
//! process_rename_city@00944090` (item 4268, `docs/CLIENT_INPUT.md` §21).
//!
//! The arm takes `objects[who][o]`; when its object flag `0x20` holds (a
//! building), it assigns the command's 22-unit name to `City +0x90`, the
//! `name` of the city the building's `BuildData +0x72` names, and then asks
//! the hot-key groups to redraw a group that holds the object (the
//! interface's, not the simulation's). It answers `0x35`, the command's
//! size. `CityData::log_data@004895c0` prints the name bare in the `CITY`
//! record, after `who`, which is the diff `rondata`'s renamed playback
//! reads.
//!
//! The same field holds the name a city is given at its founding
//! (`City::generate_name@00735c90`, the tribe's list;
//! `crate::leaders::words`, item 4418), so a rename overwrites it.

use crate::{Player, Sim};

impl Sim {
    /// `process_rename_city`: the name of the city building `(who, o)`
    /// belongs to becomes `name`. A unit, a building in no city or an
    /// object not in the simulation writes nothing. Returns whether a
    /// name was written.
    pub fn process_rename_city(&mut self, who: Player, o: i16, name: &str) -> bool {
        let Some(c) = self
            .building_by_o(who, o)
            .and_then(|b| self.buildings[b].city)
        else {
            return false;
        };
        self.cities.names.insert(c, name.to_string());
        true
    }

    /// `City +0x90` for city `c`: its founding name, or what a rename
    /// last wrote. `None` for a city no founding named (a capture, an
    /// exempt arm, or a tribe whose list was never loaded).
    pub fn city_name(&self, c: usize) -> Option<&str> {
        self.cities.names.get(&c).map(String::as_str)
    }
}
