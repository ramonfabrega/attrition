//! **Tribute and the deal record** — item 4159's step VII-b2
//! (`docs/LEADERS.md` §8): the per-partner `Diplomacy` record
//! (`LeaderData::dip`, `+0x692c`), the offer and its withdrawal
//! (`Leader::action_offer@006d1780`, `Leader::action_clear_all@006d15e0`,
//! `Leader::clear_agree@006d1af0`), the answer that commits and exchanges
//! (`Leader::action_respond@006d03c0`), the receiver's stamps
//! (`Leader::consider_tribute@006d1360`), the giver's scale
//! (`LeaderData::scale_tribute@006d5240`) and the reader that changes a
//! decision (`LeaderData::is_tribute_period@006d5b90`).
//!
//! The leaves: `tribute_stamp` (`+0x174`), `gift_stamp` (`+0x194`),
//! `counteroffer` (`+0x314`), `tribute_demanded` (`+0x334`), `tributes`
//! (`+0x498`), `tribute_sent` (`+0x860`) and `tribute_received` (`+0x864`).
//! The deal record sits past `LeaderData::walk_data`'s range, so it is on
//! no checksum line.

use crate::economy::{RESOURCES, Resource};
use crate::{Player, Sim};

/// `TAUNT_RECEIVER_STOCK`: `consider_tribute`'s `bucket[good] < 3000`, the
/// receiver's post-credit stock under which a computer counts a gift as
/// tribute.
const TRIBUTE_STOCK: i32 = 3000;

/// `is_tribute_period`'s window: a stamp younger than 3000 frames.
const TRIBUTE_PERIOD: i64 = 3000;

/// `econ[good] & 4`: the good is under the AI's `hi` mark (`docs/AI.md`
/// §2.5), so a computer receiver wants it.
const ECON_UNDER_HI: i32 = 4;

/// `Diplomacy::num_attacks`'s slots: `attacks[8]`, one per leader slot.
const SLOTS: usize = 8;

/// `Diplomacy` (`0x5c` bytes, `LeaderData::dip[8]` at `+0x692c`): what one
/// leader proposes to one partner. A positive `offers[r]` in `A.dip[B]`
/// means "A gives B"; [`Sim::action_offer`] writes it on both records with
/// opposite signs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Diplomacy {
    /// `agree` (`+0`): 1 once this side has committed the deal.
    pub agree: i32,
    /// `any_offer` (`+4`): an offer was made since the last clear.
    pub any_offer: i32,
    /// `treaty` (`+8`): the treaty asked for, −1 for none.
    pub treaty: i32,
    /// `offers[6]` (`+0xc`), by good.
    pub offers: [i32; RESOURCES],
    /// `dows[6]` (`+0x24`): an attack request's price, paid at the commit
    /// and refunded at the exchange.
    pub dows: [i32; RESOURCES],
    /// `attacks[8]` (`+0x3c`): the slots this side asks the partner to
    /// attack. Nothing in the crate sets one (`Leader::action_attack` is
    /// not carried), so every attack arm below is read and never reached.
    pub attacks: [i32; SLOTS],
}

impl Default for Diplomacy {
    /// `Diplomacy::clear_all@0047e030`, which `Leader::init` runs on every
    /// record: `treaty = −1`, every other word 0.
    fn default() -> Self {
        Diplomacy {
            agree: 0,
            any_offer: 0,
            treaty: -1,
            offers: [0; RESOURCES],
            dows: [0; RESOURCES],
            attacks: [0; SLOTS],
        }
    }
}

impl Diplomacy {
    /// `Diplomacy::num_attacks@0047dec0`: the nonzero `attacks` slots.
    pub fn num_attacks(&self) -> usize {
        self.attacks.iter().filter(|&&a| a != 0).count()
    }
}

/// The leaves the leaders line emits: the two stamps, which move on
/// run1575's block 967. `tribute_sent` and `tribute_received` are held
/// ([`Sim::leader_tribute_leaf`] answers them) and stay off the line: their
/// only witness is run1575's packet at 1090, which follows the staged accept
/// of 1027 (`docs/DECISIONS.md` entry 132 §1), so they wait on step VII-c.
/// `counteroffer`, `tribute_demanded` and `tributes` are held and stay off
/// too: nothing our walks hold moves them (`docs/LEADERS.md` §8.5).
pub const TRIBUTE_LEAVES: [&str; 2] = ["tribute_stamp", "gift_stamp"];

/// One leader's tribute leaves and deal records (`docs/LEADERS.md` §8.1).
/// `Leader::init` zeroes the leaves and clears every record; nothing in a
/// match resets them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tribute {
    /// `dip[8]` (`+0x692c`), by partner.
    pub dip: [Diplomacy; SLOTS],
    /// `tribute_stamp` (`+0x174`), the receiver's, by giver: the frame a
    /// gift counted as tribute.
    pub tribute_stamp: [i64; SLOTS],
    /// `gift_stamp` (`+0x194`), the giver's, by receiver: the frame of the
    /// last gift.
    pub gift_stamp: [i64; SLOTS],
    /// `counteroffer` (`+0x314`), by proposer.
    pub counteroffer: [i32; SLOTS],
    /// `tribute_demanded` (`+0x334`), by target.
    pub tribute_demanded: [i32; SLOTS],
    /// `tributes` (`+0x498`), by good: what is committed and not yet
    /// exchanged, whole units.
    pub tributes: [i32; RESOURCES],
    /// `tribute_sent` (`+0x860`): whole units given, unscaled.
    pub tribute_sent: i32,
    /// `tribute_received` (`+0x864`): whole units received, after the
    /// giver's scale.
    pub tribute_received: i32,
}

impl Sim {
    /// One of [`TRIBUTE_LEAVES`] as the leaders line and the record print
    /// it: the two stamps eight words, the two totals one.
    pub fn leader_tribute_leaf(&self, who: Player, name: &str) -> Option<Vec<i64>> {
        let t = self.leaders.tribute.get(who as usize)?;
        match name {
            "tribute_stamp" => Some(t.tribute_stamp.to_vec()),
            "gift_stamp" => Some(t.gift_stamp.to_vec()),
            "tribute_sent" => Some(vec![i64::from(t.tribute_sent)]),
            "tribute_received" => Some(vec![i64::from(t.tribute_received)]),
            _ => None,
        }
    }

    /// Whether `me` and `p` are two distinct slots of the deal records.
    fn deal_pair(&self, me: Player, p: Player) -> Option<(usize, usize)> {
        let (m, x) = (me as usize, p as usize);
        let n = self.leaders.tribute.len().min(SLOTS);
        (m < n && x < n && m != x).then_some((m, x))
    }

    /// `LeaderData::type_avail(r, 1) == 4` on a good: the crate's goods
    /// availability ([`crate::economy::Holdings::available`]).
    fn deal_good(&self, who: usize, r: usize) -> bool {
        self.economy.holdings[who].available[r]
    }

    /// `LeaderData::bucket_add@0043ed10`: the bucket plus `x`, unclamped.
    fn bucket_add(&mut self, who: usize, r: usize, x: i32) {
        self.economy.ledgers[who].bucket[r] += x;
    }

    /// `bucket` and `escrow` each less `x`, floored at zero: the commit's
    /// debit.
    fn debit(&mut self, who: usize, r: usize, x: i32) {
        let l = &mut self.economy.ledgers[who];
        l.bucket[r] = (l.bucket[r] - x).max(0);
        l.escrow[r] = (l.escrow[r] - x).max(0);
    }

    /// **`Leader::action_offer@006d1780(me, p, r, amount)`**: `me` offers
    /// `p` `amount` of good `r` (a negative amount asks for it). A
    /// positive amount `me` cannot cover — `bucket[r] < offers[r] +
    /// amount` — is refused. Otherwise `any_offer` is set, both sides'
    /// agreement is withdrawn ([`Sim::clear_agree`] each way), and the
    /// amount is added to `me.dip[p].offers[r]` and taken from
    /// `p.dip[me].offers[r]`. The diplomacy window's click stamp and the
    /// refusal's sound are the interface's.
    pub fn action_offer(&mut self, me: Player, p: Player, r: Resource, amount: i32) {
        let Some((m, x)) = self.deal_pair(me, p) else {
            return;
        };
        let g = r.index();
        if amount > 0
            && self.economy.ledgers[m].bucket[g] < self.leaders.tribute[m].dip[x].offers[g] + amount
        {
            return;
        }
        self.leaders.tribute[m].dip[x].any_offer = 1;
        self.clear_agree(m, x);
        self.clear_agree(x, m);
        self.leaders.tribute[m].dip[x].offers[g] += amount;
        self.leaders.tribute[x].dip[m].offers[g] -= amount;
    }

    /// **`Leader::action_clear_all@006d15e0(me, p)`**: both sides'
    /// agreement withdrawn, then both records cleared
    /// (`Diplomacy::clear_all`). The console's "proposal withdrawn" line is
    /// the interface's.
    pub fn action_clear_all(&mut self, me: Player, p: Player) {
        let Some((m, x)) = self.deal_pair(me, p) else {
            return;
        };
        self.clear_agree(m, x);
        self.clear_agree(x, m);
        self.leaders.tribute[m].dip[x] = Diplomacy::default();
        self.leaders.tribute[x].dip[m] = Diplomacy::default();
    }

    /// **`Leader::clear_agree@006d1af0(me, p)`**: when `me` has committed
    /// to `p` (`agree == 1`), what the commit moved into `tributes` goes
    /// back to the bucket, per good — first `min(tributes, dows)`, then, for
    /// a positive offer, `min(tributes, offers)` — the escrow is not
    /// refunded, and `dows` is cleared. Then `agree` and `any_offer` are 0.
    fn clear_agree(&mut self, m: usize, x: usize) {
        if self.leaders.tribute[m].dip[x].agree == 1 {
            let d = self.leaders.tribute[m].dip[x];
            for r in 0..RESOURCES {
                for (cap, live) in [(d.dows[r], d.dows[r] != 0), (d.offers[r], d.offers[r] > 0)] {
                    if live {
                        let t = &mut self.leaders.tribute[m].tributes[r];
                        let k = (*t).min(cap);
                        *t -= k;
                        self.bucket_add(m, r, k);
                    }
                }
            }
            self.leaders.tribute[m].dip[x].dows = [0; RESOURCES];
        }
        let d = &mut self.leaders.tribute[m].dip[x];
        d.agree = 0;
        d.any_offer = 0;
    }

    /// **`Leader::action_respond@006d03c0(t, p, code)`** (`docs/LEADERS.md`
    /// §8.2): `t` answers `p`'s proposal with 0 (withdraw), 1 (accept) or
    /// any other code (reject). Every code first zeroes
    /// `t.tribute_demanded[p]` and `t.counteroffer[p]`.
    ///
    /// - **0**: `t`'s agreement withdrawn ([`Sim::clear_agree`]).
    /// - **1**: the commit, unless `t` has committed already — per good
    ///   both can use, a purse short of the give plus an attack request's
    ///   price refuses the whole answer; then each positive give leaves
    ///   `t`'s bucket and escrow for `tributes`, each price likewise into
    ///   `dows`, `agree = 1`, and **`p.agendas[t] |= 4`**, which forces
    ///   `p`'s next `Leader::diplomacy` pass at `t`. An ally named in the
    ///   attack requests clears both records. Then the deal executes when
    ///   `p` has agreed, or when it asks no treaty, no attack and nothing
    ///   of `p` (a pure gift); else it waits for `p`. The exchange: each
    ///   side's `dows` back to its own bucket, then per good each side's
    ///   positive give credited to the other through the giver's
    ///   [`Sim::scale_tribute`], counted in `tribute_sent` and the other's
    ///   `tribute_received`, and stamped by [`Sim::consider_tribute`]; both
    ///   records cleared.
    /// - **Reject**: both sides' agreement withdrawn and both records
    ///   cleared.
    ///
    /// SEAM: the treaty arm (a treaty asked: the relation set both ways,
    /// `made_peace` on a peace, the neutral propagation), the alliance's
    /// team gate (`num_team_members`, `num_allies`) and the hires (an
    /// attack request: `hire_stamp`, `hire_who` and a declaration for each
    /// party) are steps VII-c's and VII-d's. Nothing in the crate asks a
    /// treaty or an attack in a deal record (`Leader::action_treaty@006d1570`
    /// is not carried, nor the leader's attack request), so no caller
    /// reaches them; the reject's human-ally attack arm likewise. The
    /// messages and sounds are the interface's.
    pub fn action_respond(&mut self, t: Player, p: Player, code: i32) {
        let Some((m, x)) = self.deal_pair(t, p) else {
            return;
        };
        self.leaders.tribute[m].tribute_demanded[x] = 0;
        self.leaders.tribute[m].counteroffer[x] = 0;
        match code {
            0 => self.clear_agree(m, x),
            1 => self.respond_accept(m, x),
            _ => {
                self.clear_agree(m, x);
                self.clear_agree(x, m);
                self.leaders.tribute[x].dip[m] = Diplomacy::default();
                self.leaders.tribute[m].dip[x] = Diplomacy::default();
            }
        }
    }

    /// [`Sim::action_respond`]'s accept: the commit, the wait and the
    /// exchange.
    fn respond_accept(&mut self, m: usize, x: usize) {
        if self.leaders.tribute[m].dip[x].agree != 1 && !self.commit(m, x) {
            return;
        }
        let d = self.leaders.tribute[m].dip[x];
        // An ally of either side named in the attack requests: both sides
        // withdrawn and both records cleared.
        let players = self.leaders.tribute.len().min(SLOTS);
        for k in (0..players).filter(|&k| k != m && k != x) {
            if d.attacks[k] != 0
                && (self.is_ally(m as Player, k as Player)
                    || self.is_ally(x as Player, k as Player))
            {
                self.clear_agree(m, x);
                self.clear_agree(x, m);
                self.leaders.tribute[m].dip[x] = Diplomacy::default();
                self.leaders.tribute[x].dip[m] = Diplomacy::default();
                return;
            }
        }
        // The wait: `p` has not agreed, and the deal asks a treaty, an
        // attack or a good of `p`.
        let theirs = self.leaders.tribute[x].dip[m];
        if theirs.agree != 1
            && (d.treaty != -1 || d.num_attacks() != 0 || theirs.offers.iter().any(|&o| o > 0))
        {
            return;
        }
        // SEAM: `treaty == 2` runs the alliance's team gate first
        // (`LeaderData::num_team_members`, `num_allies`; step VII-d); no
        // caller sets a treaty, so the exchange is taken.
        self.exchange(m, x);
        // SEAM: the treaty arm and the hires (step VII-c), unreachable with
        // no `treaty` and no `attacks` set.
        self.leaders.tribute[m].dip[x] = Diplomacy::default();
        self.leaders.tribute[x].dip[m] = Diplomacy::default();
    }

    /// The commit of [`Sim::action_respond`]'s accept: `false` when the
    /// purse refuses it.
    fn commit(&mut self, m: usize, x: usize) -> bool {
        let d = self.leaders.tribute[m].dip[x];
        let human = self.cities.nation[m].human;
        let players = self.leaders.tribute.len().min(SLOTS);
        let mut price = [0i32; RESOURCES];
        let mut give = [0i32; RESOURCES];
        for r in 0..RESOURCES {
            if !self.deal_good(m, r) || !self.deal_good(x, r) {
                continue;
            }
            give[r] = d.offers[r].max(0);
            // An attack request's price: the Declare War craft on the
            // named slot, doubled for an ally, and only a human pays it
            // (`leader_flags & 4`). The last slot named sets it.
            for k in 0..players {
                if d.attacks[k] != 0 && !self.is_enemy(m as Player, k as Player) {
                    let cost = self.dow_price(
                        m as Player,
                        crate::leaders::declare::DECLARE_WAR + k,
                        Resource::ALL[r],
                    );
                    let ally = i32::from(self.is_ally(m as Player, k as Player));
                    price[r] = cost * (ally + 1) * i32::from(human);
                }
            }
            if self.economy.ledgers[m].bucket[r] < price[r] + give[r] {
                return false;
            }
        }
        for (r, &o) in give.iter().enumerate() {
            if self.deal_good(m, r) && self.deal_good(x, r) && d.offers[r] > 0 {
                self.debit(m, r, o);
                self.leaders.tribute[m].tributes[r] += o;
            }
        }
        for (r, &c) in price.iter().enumerate() {
            if c != 0 {
                self.debit(m, r, c);
                self.leaders.tribute[m].tributes[r] += c;
                self.leaders.tribute[m].dip[x].dows[r] = c;
            }
        }
        self.leaders.tribute[m].dip[x].agree = 1;
        // `leaders[p].agendas[t] |= 4` (`6d088c`): the proposer's next
        // `Leader::diplomacy` pass at `t` runs off its cadence.
        if let Some(a) = self.leaders.agendas.get_mut(x).and_then(|r| r.get_mut(m)) {
            *a |= 4;
        }
        true
    }

    /// The exchange of [`Sim::action_respond`]'s accept, `t = m` and `p =
    /// x`.
    fn exchange(&mut self, m: usize, x: usize) {
        for r in 0..RESOURCES {
            // Each side's attack price back to its own bucket. The original
            // zeroes `t.dip[p].dows[r]` and then `p.dip[p].dows[r]`, its own
            // slot (0 over 0), not `p.dip[t]`; `Diplomacy::clear_all` below
            // clears it anyway.
            let back_t = self.leaders.tribute[m].dip[x].dows[r];
            let back_p = self.leaders.tribute[x].dip[m].dows[r];
            self.bucket_add(m, r, back_t);
            self.bucket_add(x, r, back_p);
            self.leaders.tribute[m].dip[x].dows[r] = 0;
            self.leaders.tribute[x].dip[x].dows[r] = 0;
            if !self.deal_good(m, r) || !self.deal_good(x, r) {
                continue;
            }
            let g_t = self.leaders.tribute[m].dip[x].offers[r].max(0);
            let g_p = self.leaders.tribute[x].dip[m].offers[r].max(0);
            let s_p = self.scale_tribute(x as Player, g_p);
            self.bucket_add(m, r, s_p);
            self.leaders.tribute[x].tributes[r] -= g_p;
            self.leaders.tribute[x].tribute_sent += g_p;
            self.leaders.tribute[m].tribute_received += s_p;
            self.consider_tribute(m, x, g_p, r);
            let s_t = self.scale_tribute(m as Player, g_t);
            self.bucket_add(x, r, s_t);
            self.leaders.tribute[m].tributes[r] -= g_t;
            self.leaders.tribute[m].tribute_sent += g_t;
            self.leaders.tribute[x].tribute_received += s_t;
            self.consider_tribute(x, m, g_t, r);
            for w in [m, x] {
                let t = &mut self.leaders.tribute[w].tributes[r];
                *t = (*t).max(0);
            }
        }
    }

    /// **`Leader::consider_tribute@006d1360(rcv, giver, amount, good)`**,
    /// after the receiver's credit: nothing for `amount <= 0`; else the
    /// giver's `gift_stamp[rcv]` is this frame, and the receiver's
    /// `tribute_stamp[giver]` too when the receiver is human, or when the
    /// gift is at least `100 · giver.blacken`, the receiver holds under
    /// 3000 of the good, and the good is wealth or one the receiver's AI
    /// wants (`econ & 4`).
    fn consider_tribute(&mut self, rcv: usize, giver: usize, amount: i32, good: usize) {
        if amount <= 0 {
            return;
        }
        let frame = self.frame;
        self.leaders.tribute[giver].gift_stamp[rcv] = frame;
        let blacken = self.leaders.declare.get(giver).map_or(0, |d| d.blacken);
        let counts = self.cities.nation[rcv].human
            || (blacken * 100 <= amount
                && self.economy.ledgers[rcv].bucket[good] < TRIBUTE_STOCK
                && (good == Resource::Wealth.index()
                    || self.ai.ai[rcv].econ[good] & ECON_UNDER_HI != 0));
        if counts {
            self.leaders.tribute[rcv].tribute_stamp[giver] = frame;
        }
    }

    /// **`LeaderData::scale_tribute@006d5240(giver, x)`**: what of a gift
    /// of `x` gets through. `pct = epoch[Commerce] · COMMERCE_TRIBUTE +
    /// BASE_TRIBUTE`, at least 1; at 100 or over the whole `x`; else
    /// `(x · pct + c) / 100`, with `c` 0 under 20, 50 under 100, else 99.
    pub fn scale_tribute(&self, giver: Player, x: i32) -> i32 {
        let t = &self.tables.tuning;
        let commerce = self
            .tech
            .get(giver as usize)
            .map_or(0, |p| p.epoch[crate::tech::Line::Commerce.index()]);
        let pct = (commerce * t.commerce_tribute + t.base_tribute).max(1);
        if pct >= 100 {
            return x;
        }
        let c = if x < 20 {
            0
        } else if x < 100 {
            50
        } else {
            99
        };
        (x * pct + c) / 100
    }

    /// **`LeaderData::is_tribute_period@006d5b90(me, p)`**: whether `me`
    /// holds a fresh tribute from `p` or `p`'s side. Over the slots `i` in
    /// play but `me`, the first that qualifies — `i == p`, or `i` allied
    /// with `p` both ways — decides on `me.tribute_stamp[i]`: 0 answers no,
    /// younger than 3000 frames answers yes, an older stamp lets the walk
    /// go on.
    ///
    /// SEAM: at frame 0 the original qualifies `i` by an equal lobby team
    /// in 0..3 rather than the alliance; every stamp is 0 at frame 0, so
    /// the answer is no either way. Team style 7's neutral (team 8) skip is
    /// not carried: the crate keeps no lobby team.
    pub fn is_tribute_period(&self, me: Player, p: Player) -> bool {
        let (m, x) = (me as usize, p as usize);
        let Some(stamps) = self.leaders.tribute.get(m).map(|t| t.tribute_stamp) else {
            return false;
        };
        if self.frame == 0 {
            return false;
        }
        let players = self.leaders.tribute.len().min(SLOTS);
        for (i, &stamp) in stamps.iter().enumerate().take(players) {
            if i == m {
                continue;
            }
            if i == x || self.is_ally(i as Player, p) {
                if stamp == 0 {
                    return false;
                }
                if self.frame - stamp < TRIBUTE_PERIOD {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests;
