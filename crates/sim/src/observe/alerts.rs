//! **The alert stream** (item 4605; `docs/RENDERER.md` §3.34): what the
//! original tells a player's message window from inside the simulation,
//! `MessageWin::add_message@007e9fb0` and `add_event@007e9b70` called by
//! the sim's own functions, kept as a log a client reads through
//! [`Sim::observe_alerts`] and the minimap pings (`WorldMap::render_ping`).
//!
//! **Write-only.** Nothing in the simulation reads [`Alerts`]: it is
//! appended where the original calls the message window and copied out by
//! the read door. It is in no checksummed record and no digest, and
//! `soak`'s `alerts_leave_the_digest_alone` plays a game with the log
//! cleared every frame against one with it kept.
//!
//! The original's window belongs to one console (`MiscAccess::console`),
//! and every writer asks `console->who` before it calls. Our sim has no
//! console, so an alert carries the mask of the players the writer would
//! tell ([`Alert::told`]); a client keeps the ones whose mask holds its
//! own player.
//!
//! The writers carried, each with its gate:
//! - *The attack alert, `ping` -10* (`Object::do_damage@0064a480`), on the
//!   leader and city stamps [`Sim::city_alarm`] already writes: told the
//!   victim when its stamp crosses the 300-frame window, and on the ally
//!   step the victim's allies.
//! - *The city lost or won, -6 and -7* (`do_damage`'s tail, `0064c7xx`):
//!   on `reduce_stamp`'s write, -6 to the victim and -7 to the striker.
//! - *The wonder begun, -8* (`Build::start@006273a0:137`, `announce`, the
//!   construction's call): a visible wonder told everyone, in the owner's
//!   neon. *The wonder finished, -8* (`Build::activate@00623e20:1782`):
//!   told everyone, white for its owner and the owner's neon to the rest.
//! - *The nuke*: its launch, -1 at the silo (`Build::do_missile_launch@00622670:74`);
//!   its circle, -4 at the landing (`Ammo::init@0067bbf0:652`); its
//!   impact, -1 at the landing (`Ammo::do_damage@00678060:288`). Told
//!   everyone.

use crate::{Player, Sim};

/// Every player's bit: an alert the writer tells whatever console reads it.
pub const EVERYONE: u16 = 0xff;

/// `add_message`'s and `add_event`'s default `ping_radius`.
pub const RADIUS: i32 = 0xc00;

/// The colour an alert is laid in: the writer's `WHITE`, or a player's
/// `TeamColor::get_neon`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tint {
    White,
    Neon(Player),
}

/// One `MessageWinEntry` as the sim raises it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Alert {
    /// The frame it was raised on (`MessageWinEntry +0xc`).
    pub frame: i64,
    /// The original's `ping` code: -10 attack, -8 wonder, -9 rare, -6
    /// city lost, -7 city won, -4 circle, -1 default.
    pub ping: i32,
    /// The world point (`+0x4`, `+0x8`).
    pub x: i32,
    pub y: i32,
    /// The players told, one bit each.
    pub told: u16,
    pub tint: Tint,
    /// The writer's `ping_times` argument as passed: negative is a multiple
    /// of the profile's `message_length` (`add_message`: -1 one, `-n`
    /// `(1 + n)/2`), which the client resolves; a -4's is frames.
    pub times: i32,
    /// `ping_radius` (`+0x14`).
    pub radius: i32,
}

impl Alert {
    pub fn tells(&self, p: Player) -> bool {
        p < 16 && self.told & (1 << p) != 0
    }
}

/// The log: every alert raised, in the order raised.
#[derive(Clone, Debug, Default)]
pub struct Alerts {
    pub log: Vec<Alert>,
}

impl Sim {
    /// Appends one alert at the frame. The only writer of [`Alerts`].
    pub(crate) fn alert(&mut self, ping: i32, at: crate::Pos, told: u16, tint: Tint, times: i32) {
        self.alert_with_radius(ping, at, told, tint, times, RADIUS);
    }

    pub(crate) fn alert_with_radius(
        &mut self,
        ping: i32,
        at: crate::Pos,
        told: u16,
        tint: Tint,
        times: i32,
        radius: i32,
    ) {
        if told == 0 {
            return;
        }
        let frame = self.frame;
        self.alerts.log.push(Alert {
            frame,
            ping,
            x: at.x,
            y: at.y,
            told,
            tint,
            times,
            radius,
        });
    }

    /// The bit of each player allied to `p`, `p` itself left out: the
    /// ally steps' `param_2 != console && is_ally(victim, console)`.
    pub(crate) fn allies_mask(&self, p: Player) -> u16 {
        (0..8u8)
            .filter(|&q| q != p && self.is_ally(p, q))
            .fold(0, |m, q| m | 1 << q)
    }

    /// **The read door's copy of the log** from its `from`th alert on: a
    /// client keeps the count it has read and asks for the rest.
    pub fn observe_alerts(&self, from: usize) -> Vec<Alert> {
        self.alerts.log.get(from..).unwrap_or_default().to_vec()
    }
}
