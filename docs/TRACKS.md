# The tracks

The finish line, decomposed into the avenues the loop can run, each with
the number a landing moves and the gate that says the move was real. The
steering pass maintains this page (`docs/LOOP.md`, "the passes"); the
queue books from it; no backlog file stands under it (`docs/DECISIONS.md` entry 125). A fresh
session reads the queue first and this page second, to know which track
its item serves. Written at the seventy-fourth pass, 2026-11-08, with Ramon
on the steer row (`docs/DECISIONS.md` entry 114); the milestone and the
order of play are the seventy-sixth's, 2026-11-09, also with Ramon (entry
116).

## The finish line (decided with Ramon, the seventy-fourth pass)

Our own app, no Wine. It opens in a moment. A lobby asks for a map, a
nation and computer opponents; one click, and the game is on screen fast,
drawn with the original's art, running smooth and light. You play a whole
game against the computer. Under the picture the rules are the original's
exactly: fed the same orders, Rise of Nations does the same thing frame for
frame. A game can be saved and replayed, by us and by the original.

**What "exactly" is counted in** (entry 132 §10, Ramon, 2026-11-13): zero
mismatched frames on the twelve state checksums over a declared set of
games. The original computes them every turn and prints them only in a dump
window; step H (item 4214) hooks them every frame. Until it prints, the
draw stream's floors stand in, and they are blind to state: two files three
units apart agree on every draw of 24,001 frames.

Not in it, by design: multiplayer, Conquer the World, the editor, new art.
Each is a finish line of its own, after this one is finished and tested.

## The milestone: one lobby, all the way through

**A game played here, on a start built here, is replayed by the original and
agrees with it for its full length.** One lobby — Great Lakes, the harness's
own — taken through every track it needs before any track goes wide:

1. **The start, built** from the lobby's choices with no dump of the
   original (`Start: 1 of 161`; 161 of 161 since item 4211).
2. **The game run from that start** in lockstep with the lobby's long
   capture, where it ran from an installed dump before.
3. **A recorded game written** in the original's container, and the
   original playing a file we wrote (`docs/RECGAME.md`, the playback
   preflight).
4. **Orders minted through `Command`** — a script first, the client's mouse
   after — carried in that file.
5. **The checksum**, ported, agreeing on the frames the playback prints.

Once a person plays, no capture of the original exists to diff against. The
original replaying our game is the only oracle left, and it scales to every
game anyone plays afterwards. So this is reached before the start block is
taken across the other 160 lobbies, before M is counted on a second lobby,
and before the census filler: breadth on a track is the filler, and the
chain is the headline. The steps are booked in `docs/QUEUE.md`; the driver
designs each as the one before it lands.

**Closed at the pass of 2026-11-18, second** (entry 143 §1): all five
steps, step 5 ratified with its limits named. The headline since is the
finish line's own counter, parity on the declared set (entry 143 §2).

## The tracks

| track | what "done" is | the number | its gate | state at the pass of 2026-11-14 (earlier states in the passes' records under `docs/audit/`) |
|---|---|---|---|---|
| **rules** (phase 3) | every captured game replays in lockstep for its full length | the scoreboard's words, `rondata::diff`'s floors; entry 29's three counters | the release diff suite | **whole** (`Scoreboard:`, `Golden:`): every word at its length, 130 chapters closed; no item stands; a chapter is booked only when another track's frame needs one |
| **structure** | the sim crate has a public surface and subsystem boundaries, the tests and sweeps live outside the source, nothing is defined twice | `docs/STRUCTURE.md`'s shape counts (files in the flat root, lines of test code in `src/`, duplicated names, the `Sim` struct's fields), each falling to its target; **every floor unchanged on every landing** | the whole release gate, no behaviour diff | every target of `docs/STRUCTURE.md` met (`Structure:`), and a gate on every later move; item 4224, one walk a game, is booked under its kind, whole |
| **renderer** (phase 4) | a captured game drawn with the original's art, placed where the original would draw it | pieces in register K of M (`docs/RENDERER.md` §5.4); frames in register as its floor; the clock: seconds to lobby, seconds to frame zero, slowest-percentile frame milliseconds, peak resident memory | a `render` crate's pins, parsed as the queue's | `Renderer:` K 59 of M 60 (4208's birds), frames 35 of 35; the rows read the client's still (4171), and the ground, the coast, the land's light and tint, the trees, the ring, the paved base and the trees' shadows have landed on it (4173 to 4223); the lake lit in HDR with its surf (4232 to 4239) and the fire's particles, light and bloom (4233 to 4241); the pass of 2026-11-14, second, looked and booked the buildings' colour, the mountain and the map's edge (4245 to 4247) |
| **start** | a game starts from a lobby's choices with no dump of the original: map generation's remaining chain and the frame-zero start builder | the start block equal to the original's on every lobby on disk (the dumps print it) | `rondata::diff` on the start blocks | **closed**: `Start: 161 of 161 lobbies` (4034, 4186, 4196, 4211), the harness's lobby walked from it for 24,000 frames |
| **checksum** | the original's full-state hash, ported, compared per frame against the original's own | ~~frames whose hash agrees, of the frames the original prints under `CHECKSUM=2`~~ the state lines that agree, of twelve, on the frames a `DUMP_ALL` window of the playback prints (entry 119: that setting prints no frame) | the diff suite | **open**, the milestone's step 5 (`Checksum:`): state 8 of 12 lines, lines 422 of 1,053; the leaders line the last that counts, `LEADERS_HELD_OUT` 23 (item 4159); parity 0 of 1 games, step H printing every frame (4214, run1578), its first partings units 6907, guys 7273, builds 7383, groups 8206 (items 4243, 4244) |
| **orders** | a mouse and keyboard mint the same order stream the replays carry; the stream is the replay and, later, the network format | orders issued through the public `Command` API replaying in lockstep | the diff suite on a minted stream | **step 4 closed** (`Recording:`): clicks, right presses, a box, an attack and a follow, each minted here and played by the original (4149 to 4202); item 4249 counts the order kinds the client mints, toward a game a person plays (entry 134 §11) |
| **lobby** | our own screen producing the start's choices | a page of client code; no counter | the client's tests | `Lobby:` 161 of 161; a plain HUD, the menu and the panel (`Hud:`; 4197, 4198, 4203) |
| **AI** (phase 5) | a computer opponent plays a whole game against a person in our client | the held-out battery, games nobody tuned for scored on their first try; then the original replaying games played here against the computer | the diff suite; the original on the capture lane | **ported as far as the captures reach**: the computer's decisions are computed here, not replayed, in every lockstep game on the scoreboard, at four difficulties (`Held-out:` 8 of 9 whole). What remains is the branches no capture entered, which a person's play will enter and the milestone's oracle will score |
| **save and replay** | a game saved as seed plus orders, replayable here and written back into the original's recorded-game container | a saved game replays to the same hash; the original replays one of ours | the diff suite; the original on the capture lane | **step 3 stands** (`Recording:`): every recording on disk round-trips, and the original plays our order-free file for 24,000 of 24,000 frames (4058, run1560) |
| **census** | the territory count tells the truth about the original's 3,611 simulation functions | ported · replaced by design · open, the readers' marks landed row by row (`docs/audit/2026-11-06-territory-*.md`; lore's scoring of the `replaced` groups) | `tools/census.py`'s pins | `Rubric:` 574 open of 3,611, 569 of them carrying a reader's ruling (entry 132); filler for an idle lane |

**At the pass of 2026-11-18** (entry 142): **checksum** — parity reads 5 of 6 games: the minted game and four live windows the client
recorded and the original played back (Codex, items 4362 to 4373), two with combat, so the ammo and deaths records are exercised; the sixth, the
2017 sample, parts on state 0 on four lines and is read from there (item 4384); `LEADERS_HELD_OUT` 1, the demand's walk at 5677 of 6,081 after
ten lanes, and the leaf is now tried on a small game our walk holds, found by a sweep in our own sim (item 4382); **orders** — every input
kind is minted and the blind list is closed; the Repair pick's order is the last unplayed arm (item 4383); **renderer** — the settled border's
band is blended in HDR and reads theirs' lavender; its row is re-fit to read by value (item 4385), and the seat then returns to the lake by the card's own draws (item 4386; entry 142 §7).

**At the pass of 2026-11-18, second** (entry 143): **the milestone is closed**; **checksum** — step 5 ratified, nine of twelve lines
on frame 91 and all twelve on every frame of the minted game; parity 5 of 6, the sample's state 0 at 3 parted ranges (item 4397); the
demand's game, whole on 6,401 states, is played to its end, which ours reaches on 10929 (item 4396); the packets' lines, 608 of 1,053,
are booked behind them (item 4400); **orders** — whole by its counters; the seat draws what a selected city shows (item 4398);
**renderer** — three lanes on the lake's streaks moved nothing, the streaks differ between takes, and the seat scores what two takes
agree on (item 4399).

**At the pass of 2026-11-19** (entry 144): **checksum** — parity 6 of 7, the demand's game ratified as the seventh with its end tick
pinned; the sample's lowest first parting 532 of 21,044 (item 4414); the packets' lines 690 of 1,053 (items 4415, 4417); a game is
declared before the original plays it, and parity held out is booked on eight lobbies nobody tuned for (item 4413); **orders** — whole
by its counters; a live game shows no city name, and the third seat ports the founding name (item 4416); **renderer** — the selected
city's band, plate and bar are drawn; the lake has no seat, three takes of one state agreeing on 3.8 % of it.

**At the pass of 2026-11-19, second** (entry 145): **checksum** — parity 6 of 7; parity held out ratified at 0 of 8 as found, 64 of 96
lines, and parsed on the scoreboard: the four Ancient starts are near whole and the four later starts part for good by 2239 and are ended
by the original (items 4435, 4436, each game's ranges taken once); a held-out game is spent once a lane reads it, and a second battery
is named; the sample's lowest first parting 2700 of 21,044 (item 4432); the packets' lines 771 of 1,053 (items 4430, 4431); **orders** —
whole by its counters; **renderer** — the plate reads the city's name in the install's face; the client opens every lobby on Great
Lakes' camera (item 4437), and theirs' buildings carry a team colour ours lack (item 4438).

**At the pass of 2026-11-20** (entry 146): **checksum** — parity 6 of 7; parity held out 3 of 8 now, 78 of 96 lines (0 of 8 and 64 as
found); the pass scored the rows with ranges on disk by leaf, and rows 80 and 100 part builds on one bit, row 80 on nothing else (item
4455); row 140 is three states from whole and takes its ranges (item 4457); the sample's lowest first parting 3878 of 21,044 (item
4456); the packets' lines 817 of 1,053 (items 4461, 4431); the second battery is booked, its declaration first (item 4462); whether the
eight fold into the headline is Ramon's, recommended, 9 of 15 folded; **orders** — whole by its counters; **renderer** — the client opens
on slot 0's capital on 161 of 161 lobbies and the buildings carry the team's colour; theirs fades a wreck under dust and ours draws it
solid (item 4458).

**At the pass of 2026-11-20, second** (entry 147): **checksum** — parity 6 of 7, parsed on the scoreboard as `Parity:` with the
sample's lowest first parting, 3921 of 21,044 (item 4479); parity held out 5 of 8 now, 89 of 96 lines, and row 60 is one line and 91
states from whole (item 4480); the second battery read 1 of 8 and 65 of 96 as found, 2 of 8 now, and by reach, the states an unseen
game runs before its first parting, 53 % against the first's 27 %; its rows 90 and 70 are a state and a line from whole (items 4470,
4486); a third battery is named (item 4487); the packets' lines 853 of 1,053 (item 4481); the fold is Ramon's, recommended, 11 of 15
folded; **orders** — whole by its counters; **renderer** — the wreck was three live tanks under a haze no row counts (item 4482).

**At the pass of 2026-11-21** (entry 148): **checksum** — parity 6 of 7; the sample's lowest first parting 4248 of 21,044 and
its draw word 5273, both on `Parity:` now; the word moved 945 frames in four landings while the score stood, so the sample's seat
takes ranges past the word first (item 4505); parity held out 7 of 8 now, the second battery 4 of 8, and the third read 5 of 8 and
90 of 96 lines as found, 88 % by reach, 6 of 8 now; rows 40 and 5 are one line from whole (items 4484, 4507), rows 105 and 10 hold
eight leaves on disk (items 4502, 4504); a row's take keeps the lines its score parts on, because the disk reads 75 GB against a
floor of 60; a fourth battery is named (item 4510); the packets' lines 919 of 1,053; the fold is Ramon's, recommended, 13 of 15
folded; **orders** — whole by its counters; **renderer** — the haze is a moving vehicle's dust, an emitter the view starts, and the
eye's seat draws it from the packet's emitters (item 4506).

**At the pass of 2026-11-21, second** (entry 149): **checksum** — parity 6 of 7; the sample parts first on world 5401 of 21,044,
draws 5460, and its next take runs to 6400 on the ten lines that part below it (item 4526); parity held out 8 of 8 now, the second
battery 6 of 8, the third 8 of 8, and the fourth read 5 of 8 and 90 of 96 lines as found, 91 % by reach; rows 75, 30, 55, 95 and
110 are scored and each needs a take (items 4517, 4509, 4518, 4519, 4485); the disk no longer holds the takes: the archive's
compression skipped every file of 2^31 bytes, thirteen ranges records are kept in gigabyte pieces, and 56 GB free became 117 with
nothing deleted; a fifth battery is named (item 4527); the packets' lines 948 of 1,053; the fold is Ramon's, recommended, 14 of 15
folded; **orders** — whole by its counters; **renderer** — the tanks' dust is drawn from the packet, 4 of 4 (item 4506).

**At the pass of 2026-11-22** (entry 151): **checksum** — parity 39 of 47, the five spent batteries folded (37 of 39 on the same
pins before the fifth joined: it was spent at item 4534 and a guard now reads the spent list); the sample parts first on units 6108
of 21,044, draws 6114, and its seat, stopped against its rule, reopens (item 4543); parity held out 8, 7, 8, 8 and 2 of 8 now; the
fifth, three seats and a team a row, read 1 of 8 and 30 of 96 lines as found, 48 % by reach where the fourth read 91; rows 110, 63,
103, 83, 123, 43 and 3 are scored (items 4541, 4542, 4544 to 4548), row 110 three leaves from whole and each fifth row one take;
a sixth battery is named, taken when the fifth reads 4 of 8; the packets' lines 950 of 1,053; **orders** — whole by its counters;
**renderer** — as it stood, no landing.

**At the pass of 2026-11-22, second** (entry 152): **checksum** — parity 41 of 47 (rows 110 and 123 whole); the sample parts first on
units 7055 of 21,044, draws 7105, 158 states a landing over six and slowing; parity held out 8, 8, 8, 8 and 3 of 8 now, the fifth's
reach 81 %; five rows scored: row 63 is one field from whole (item 4563), the sample and row 43 name one pair of fields and take one
seat (items 4565, 4564), row 103's first shot is a piece with no bay and the games on disk field eight such, built in one act (item
4559); the sixth battery keeps its trigger and is recorded to 12,000, because every held-out game ends on 6,000 and the sample parts
past it, its human scripted on Ramon's word (item 4566); the packets' lines 950 of 1,053; **orders** — whole by its counters; **renderer** — as it stood, no landing.

**At the pass of 2026-11-22, third** (entry 153): **checksum** — parity 43 of 47 (rows 63 and 103 whole); the sample parts first on
leaders 7407 of 21,044, draws 7410; parity held out 8, 8, 8, 8 and 5 of 8 now, the fifth's reach 87 %; the sixth, to 12,000 with a
scripted human, read 1 of 8 and 20 of 96 lines as found, 49 % by reach and 59 % inside its first 6,001 states: six rows part before
3,400, one on 7675, one runs whole; a landing on a row that parts early bought 732 states and one deep in a game about 190, so the
seats go by first parting and the sample holds none of its own; row 3 is one line from whole (item 4572), rows 73, 33 and 113 part
units and cities on one state and take one seat (item 4576), and the sixth's walks fire eleven pieces with no bay, built in one act
(item 4574); a seventh battery is named; the packets' lines 950 of 1,053; **orders** — whole by its counters; **renderer** — as it
stood, no landing.

**The order of play** (the seventy-sixth pass, entry 116). The milestone
first, one lobby deep, on the driver's own builders; the renderer beside it
on one lane, taking textures and team colours next; then the lobby screen
and the client's input, which the milestone's steps 3 and 4 make
checkable; then breadth — the other lobbies' start blocks, M on more
lobbies, the census filler — as filler for an idle worker. The rules track
is whole and stays whole: its floors run on every gate forever.

**The workers** (entries 116, 126, 128 §4, 129 §1, 134 §2): three gating
seats since 2026-11-14, on a day's budget of about fourteen points: since
2026-11-15, with parity's four lines whole, one lane on render, one on the
client's input, and the driver's builders on the chain's leaders line. At
most one lane holds an item the queue marks whole. No lane gates whole but
a `structure` lane since entry 132 §6: a lane's gate runs its brief's
filters and the booking gate, scoped, runs the corpus. The count is
Ramon's, and a pass moves it only by recommending; he may add a fifth or
sixth mid-tranche on the booking gate's minutes.
