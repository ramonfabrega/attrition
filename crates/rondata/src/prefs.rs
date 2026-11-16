//! **The player's profile prefs, and the `leader_options` order they make**
//! (item 4289, `docs/CLIENT_INPUT.md` §26.1, §30; DECISIONS 136 §4).
//!
//! The original issues `leader_options` once, at the start of a game, from
//! the player's profile and from no button: `Game::init_rules_and_teams@00589bb0`
//! resets `MiscAccess::my_leader_option` and calls
//! `LeaderOption::synch_with_prefs@006f1b50`, and `Game::run@00584590`'s
//! `LeaderOptionOut::update@006f1a60` hands it to
//! `CommandManager::issue_leader_option@00942080`, which writes command
//! `0x49`, 0x21 bytes, into the frame-0 package. [`Prefs`] is our own record
//! of the four profile fields that order reads; [`Prefs::default`] is the
//! profile `PlayerProfile::default_profile@005ceca0` makes, which is what
//! every capture lane's `Player.dat` holds.

use crate::commands::Command;

/// The profile's `prefs` bits `synch_with_prefs` reads, each with the
/// `LeaderOption` mask bit it sets, in the function's order. The last one
/// read decides the `BitMask` `flags` word (see [`Prefs::leader_option`]).
pub const PREFS_BITS: [(u32, u8); 5] = [
    (0x4000, 1),
    (0x80000, 3),
    (0x10_0000, 4),
    (0x8000, 0),
    (0x80_0000, 5),
];

/// The four fields of `PlayerProfile::prefs` (`PlayerPrefs_u_0_s_0`, the
/// PDB's names; `Player.dat`'s `PREFS`, `PEASANT_STANCE`, `PEASANTS_WAIT`,
/// `BUILDING_STANCE`, which `PlayerProfile::load@005cb430` reads in the
/// struct's order) that `leader_options` is made of. The other bits of
/// `prefs` are carried and never read by the order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Prefs {
    /// `prefs`, `+0x0`: of it the order reads [`PREFS_BITS`] alone.
    pub prefs: u32,
    /// `peasants`, `+0x38`.
    pub peasants: i32,
    /// `peasants_wait`, `+0x3c`: the order reads only whether it is zero.
    pub peasants_wait: i32,
    /// `buildings`, `+0x40`.
    pub buildings: i32,
}

/// `default_profile`'s `prefs` word: `0`, then `default_game@005d0220`
/// (`& 0xff6efefd | 0x0d0e4045`), `default_profile`'s own mouse lines
/// (`& 0xfffff1df | 0x200000`), `default_video@005cf740`
/// (`& 0x7dbfffff | 0x2000000`) and `default_audio@005cf100`
/// (`& 0xffffff7f | 0x10001010`). The three lanes' `Player.dat` hold
/// `PREFS value="523128917"`, the same word.
pub const DEFAULT_PREFS: u32 = 0x1f2e_5055;

impl Prefs {
    /// A fresh profile (`PlayerProfile::load` finds no file and calls
    /// `default_profile`): [`DEFAULT_PREFS`], and `default_game` writes
    /// `peasants` 0, `peasants_wait` 2 and `buildings` 0. The three lanes'
    /// `Player.dat` hold `PEASANT_STANCE` 0, `PEASANTS_WAIT` 2 and
    /// `BUILDING_STANCE` 0.
    pub const FRESH: Prefs = Prefs {
        prefs: DEFAULT_PREFS,
        peasants: 0,
        peasants_wait: 2,
        buildings: 0,
    };
}

impl Default for Prefs {
    /// [`Prefs::FRESH`].
    fn default() -> Prefs {
        Prefs::FRESH
    }
}

/// `MiscAccess::my_leader_option` after `synch_with_prefs`: the
/// `LeaderOptionData` record (`who`, `peasants`, `peasants_wait`,
/// `buildings`, and the `BitMask<32>` `flags`: `bits`, `size`, its own
/// `flags` word and `ptr[4]`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeaderOption {
    /// −1, the staging sentinel `init_rules_and_teams` writes; the issuer
    /// replaces it with the console's seat.
    pub who: i32,
    pub peasants: i32,
    pub peasants_wait: i32,
    pub buildings: i32,
    /// `BitMask<32>::bits`, 0x20 (`LeaderOption::LeaderOption@006f1db0`).
    pub bits: i32,
    /// `BitMask<32>::size`, 4 bytes.
    pub size: i32,
    /// `BitMask<32>::flags`.
    pub flags: i32,
    /// `BitMask<32>::ptr`: the mask's four bytes.
    pub ptr: [u8; 4],
}

impl Prefs {
    /// `LeaderOption::synch_with_prefs@006f1b50` over the record
    /// `init_rules_and_teams` resets (`who` −1, `peasants` 0,
    /// `peasants_wait` 2, `buildings` 0, the mask cleared, bits 1 and 3
    /// set, `flags` 0). `peasants` and `buildings` are copied. Each of
    /// [`PREFS_BITS`] sets or clears its mask bit; a set bit writes `flags`
    /// 0, and a clear one writes 2 where `flags` is 0, so `flags` ends 0
    /// exactly when the last bit read (`0x800000`) is set. `peasants_wait`
    /// is 2 when the profile's is nonzero, else 0. Mask bits 2, 6 and 7 and
    /// bytes 1 to 3 stay as the reset's `memset` left them, zero.
    pub fn leader_option(&self) -> LeaderOption {
        let mut mask: u8 = 0b1010;
        let mut flags = 0;
        for (bit, m) in PREFS_BITS {
            if self.prefs & bit != 0 {
                mask |= 1 << m;
                flags = 0;
            } else {
                mask &= !(1 << m);
                if flags == 0 {
                    flags = 2;
                }
            }
        }
        LeaderOption {
            who: -1,
            peasants: self.peasants,
            peasants_wait: if self.peasants_wait != 0 { 2 } else { 0 },
            buildings: self.buildings,
            bits: 0x20,
            size: 4,
            flags,
            ptr: [mask, 0, 0, 0],
        }
    }

    /// `CommandManager::issue_leader_option@00942080` on
    /// [`Prefs::leader_option`]: command `0x49`, `who` the console's seat
    /// (`MiscAccess::console->who`, not the record's −1), the three counts,
    /// and the `BitMask` copied whole by `BitMask<8>::operator=@0042ef10`
    /// (`bits`, `size`, `flags`, then `size` bytes of `ptr`).
    pub fn issue_leader_option(&self, who: i32) -> Command {
        let o = self.leader_option();
        let mut mask = [0u8; 16];
        mask[0..4].copy_from_slice(&o.bits.to_le_bytes());
        mask[4..8].copy_from_slice(&o.size.to_le_bytes());
        mask[8..12].copy_from_slice(&o.flags.to_le_bytes());
        mask[12..16].copy_from_slice(&o.ptr);
        Command::LeaderOptions {
            who,
            peasants: o.peasants,
            peasants_wait: o.peasants_wait,
            buildings: o.buildings,
            mask,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default word is `default_profile`'s composition, line by line.
    #[test]
    fn the_default_prefs_word_is_default_profile_s() {
        let mut v = 0u32;
        v = v & 0xff6e_fefd | 0x0d0e_4045;
        v = v & 0xffff_f1df | 0x20_0000;
        v = v & 0x7dbf_ffff | 0x200_0000;
        v = v & 0xffff_ff7f | 0x1000_1010;
        assert_eq!(v, DEFAULT_PREFS);
        assert_eq!(DEFAULT_PREFS, 523_128_917, "Player.dat's PREFS");
    }

    /// The fresh profile's order: mask `0x0a`, `flags` 2, `peasants_wait`
    /// 2, and its wire bytes, 0x21 with the type.
    #[test]
    fn the_fresh_profile_s_order() {
        let o = Prefs::default().leader_option();
        assert_eq!((o.ptr, o.flags, o.peasants_wait), ([0x0a, 0, 0, 0], 2, 2));
        let bytes = crate::commands::encode(&[Prefs::default().issue_leader_option(0)]);
        assert_eq!(bytes.len(), 0x21);
        assert_eq!(
            &bytes[17..],
            &[32, 0, 0, 0, 4, 0, 0, 0, 2, 0, 0, 0, 0x0a, 0, 0, 0]
        );
    }

    /// `flags` is the last bit's: set, 0, whatever came before; clear, 2.
    #[test]
    fn the_flags_word_is_the_last_bit_s() {
        let all = PREFS_BITS.iter().fold(0, |a, b| a | b.0);
        let o = Prefs {
            prefs: all,
            ..Prefs::default()
        }
        .leader_option();
        assert_eq!((o.ptr[0], o.flags), (0b11_1011, 0));
        let o = Prefs {
            prefs: all & !0x80_0000,
            ..Prefs::default()
        }
        .leader_option();
        assert_eq!((o.ptr[0], o.flags), (0b01_1011, 2));
        let o = Prefs {
            prefs: 0,
            peasants_wait: 7,
            ..Prefs::default()
        }
        .leader_option();
        assert_eq!((o.ptr[0], o.flags, o.peasants_wait), (0, 2, 2));
        let o = Prefs {
            peasants_wait: 0,
            ..Prefs::default()
        }
        .leader_option();
        assert_eq!(o.peasants_wait, 0);
    }
}
