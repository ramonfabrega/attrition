//! **`SpellTypeData::find_target@00674920`'s steps** (item 3387): a spell's
//! target search, as a pure map from `spell_flags` to the console search's
//! arguments, then the one call. The crate has no spell-flag to search-index
//! mapping of its own (its `find_target` methods in `army.rs` and `muster.rs`
//! are namesakes: an army's and a muster's, not a spell's), and no console
//! (`Console::find_target@007cba20` is answered per row by the sweep), so
//! this is the port, written from the listing and held only by the sweep.

/// `SearchIndexBH` as the original numbers it.
pub const SEARCH_ALL: i32 = 0;
pub const SEARCH_FRIENDLY: i32 = 1;
pub const SEARCH_ALLIED: i32 = 4;
pub const SEARCH_NON_FRIENDLY: i32 = 6;

/// The console's side of the call: `Console::find_target(param_3, search,
/// flags, FILTER_ALL, 0, 0, ..)`.
pub trait SpellConsole {
    fn find_target(&mut self, param3: i32, search: i32, flags: i32) -> i32;
}

/// The search index and flag word `spell_flags` maps to.
pub fn spell_search_args(spell_flags: u32) -> (i32, i32) {
    let f = spell_flags;
    let search = if f & 0x10 == 0 {
        SEARCH_NON_FRIENDLY
    } else if f & 0x20 == 0 {
        if f & 0x80 != 0 {
            SEARCH_ALLIED
        } else {
            SEARCH_FRIENDLY
        }
    } else {
        SEARCH_ALL
    };
    let mut w = (f >> 1) & 1;
    if f & 4 != 0 {
        w |= 2;
    }
    if f & 0x100 != 0 {
        w |= 0x4000;
    }
    if f & 0x200 != 0 {
        w |= 0x8000;
    }
    (search, w as i32)
}

/// `SpellTypeData::find_target(this, _, _, param_3)`.
pub fn spell_find_target<C: SpellConsole>(c: &mut C, spell_flags: u32, param3: i32) -> i32 {
    let (search, flags) = spell_search_args(spell_flags);
    c.find_target(param3, search, flags)
}
