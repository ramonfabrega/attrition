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
(item 4574); a seventh battery is named; the packets' lines 950 of 1,053; **orders** — whole by its counters; **renderer** — no
landing; Ramon gives the fourth seat to the client, and it draws the minimap, which the window lacks (item 4581); item 4580
counts the sim's `SEAM`s the held-out games enter.

**At the pass of 2026-11-23, second** (entry 155): **checksum** — parity 48 of 55 (row 113 whole); the sample as it stood, leaders
7502, draws 7613; parity held out 8, 8, 8, 8, 6 and 4 of 8 now, the sixth's reach 61 %; the seventh, its human ordering through the
game, read 1 of 8 and 20 of 72 lines as found on six rows taken, 59 % by reach and 87 % inside its first 6,001 states, and folds at
its first leaf; eight row landings bought 163 states each on late-age rows; a row within 600 states of its end ranks with the
near-whole, and the seventh's row 28 takes the first seat (item 4613); rows 133 and 43 name one shape (item 4610); the packets'
lines 951 of 1,053; **orders** — whole by its counters; **renderer** — the minimap 3,213 of 3,600 cells (items 4596 to 4605); the
client's seat takes the fog, which no capture on file shows (item 4612); a capture is filed once.

**At the pass of 2026-11-23, third** (entry 156): **checksum** — parity 50 of 63 (the seventh folded on its eight rows; its row 28
whole, item 4630); the sample as it stood, leaders 7502, draws 7613; parity held out 8, 8, 8, 8, 6, 4 and 2 of 8 now, the seventh's
reach 60 % and 86 % inside its first 6,001 states; thirteen row landings bought 43 states each, 313 of the 553 on three that built
launch points a probe had already listed, so an item an instrument has named is seated before a row's next word: the seam
`attack_ground_aim`, entered by five open rows and no whole one (item 4608), and the pieces with no bay, 1,796 rounds on four rows
(item 4618); the near-end rank goes on its own bet; row 128 stands one line from whole (item 4635); a take in flight is not scored;
the packets' lines 951 of 1,053; **orders** — whole by its counters; **renderer** — the fog, its blips and the selection ring drawn
on run1814b (items 4612 to 4629); the client's seat takes the capital's ground, the Farm's plots and the buildings' plates (item
4636); the archives are compressed one tree at a time.

**At the pass of 2026-11-25** (entry 157): **checksum** — parity 51 of 63 (the seventh's row 128 whole, item 4635); the sample
parts first on units 10653, draws 11488 (the settlement of 2026-11-24, Astra's branch); parity held out 8, 8, 8, 8, 6, 4 and 3 of 8
now; seventeen row landings bought 9,518 states, 8,663 of them on the one game that stood a line from whole and 855 on the other
sixteen; row 148 stands one line from whole and takes the first seat (item 4637), and a guard holds the queue's unseated rows to
the seats' order; the stop counts builds, and row 93's item is one (item 4657); the siege muster's seam, entered by three open rows
before their first partings and by no whole row, is an item (4672); a lane lands on a verdict its gate printed; the packets' lines
955 of 1,053; **orders** — whole by its counters; **renderer** — the Farm's plots and the buildings' plates drawn on run1814b
(items 4636 to 4665); the client's seat takes the trees past sight (item 4670).

**At the pass of 2026-11-25, second** (entry 158): **checksum** — parity 52 of 63 (the seventh's row 148 whole, item 4637); the sample
parts first on units 10653, draws 11488; parity held out 8, 8, 8, 8, 6, 4 and 4 of 8 now; sixteen row landings bought 3,308 states, each a
build that moved its row (rows 133, 88 and 93 to 3620, 3717 and 3524), and the eleven open games stand 59,253 states short of their ends;
a row's take runs to its last state and is taken once (row 88 took six windows for six words); the pivot's seam, 65 states past row 133's
parting, takes a seat (item 4656) beside rows 93 and 133 (items 4692, 4691); the packets' lines 959 of 1,053; **orders** — whole by its
counters; **renderer** — the trees past sight, the plates and the road pieces under them drawn on run1814b (items 4670 to 4687), and the
still reads as the photograph outside the HUD; the client's seat takes run1559b's picks (item 4690), then the city's name plate (item 4694).

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
