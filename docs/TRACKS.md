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

Not in it, by design: multiplayer, Conquer the World, the editor, new art.
Each is a finish line of its own, after this one is finished and tested.

## The milestone: one lobby, all the way through

**A game played here, on a start built here, is replayed by the original and
agrees with it for its full length.** One lobby — Great Lakes, the harness's
own — taken through every track it needs before any track goes wide:

1. **The start, built** from the lobby's choices with no dump of the
   original (`Start: 1 of 161`).
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

## The tracks

| track | what "done" is | the number | its gate | state at the seventy-sixth pass (2026-11-09; earlier states in the passes' records under `docs/audit/`) |
|---|---|---|---|---|
| **rules** (phase 3) | every captured game replays in lockstep for its full length | the scoreboard's words, `rondata::diff`'s floors; entry 29's three counters | the release diff suite | **whole**: every word at its length, 130 chapters closed (134–136 closed by the shelved items on their first takes); no item stands; a chapter is booked only when another track's frame needs one |
| **structure** | the sim crate has a public surface and subsystem boundaries, the tests and sweeps live outside the source, nothing is defined twice | `docs/STRUCTURE.md`'s shape counts (files in the flat root, lines of test code in `src/`, duplicated names, the `Sim` struct's fields), each falling to its target; **every floor unchanged on every landing** | the whole release gate, no behaviour diff | **done**: every target of `docs/STRUCTURE.md` met (`flat` 11 under its ceiling of 12, `fields` 21, `names` 15, `largest` 3721, `test-tails` 0, the doors); it stays as a gate on every later move |
| **renderer** (phase 4) | a captured game drawn with the original's art, placed where the original would draw it | pieces in register K of M (`docs/RENDERER.md` §5.4); frames in register as its floor; the clock: seconds to lobby, seconds to frame zero, slowest-percentile frame milliseconds, peak resident memory | a `render` crate's pins, parsed as the queue's | **K 59 of M 59** on one lobby, frames in register 35 of 35, pieces decoding 1,465 of 1,478, first pixels on screen; the next number is candidate C's silhouettes with textures and team colours (item 4038), and M on a second lobby waits behind the milestone; **at the pass of 2026-11-12** (entry 127): candidate C whole on its photograph, 33 of 33, a colour row resting at 18, the ground 1,097 of 1,109 and 1,236 of 1,920 on two photographs, 1,167,081 of 1,171,799 objects resolved over 110 lobbies — and every photograph row counted on a CPU painter that reads no brightness, so the next numbers are those rows on the client's own still and a luminance row (items 4171, 4172), then the trees, the water and the corpses; **at the pass by supply the same day** (entry 128): those landed (4171 to 4175), run1559b's still is close to its photograph and run1564's lake is not drawn as water; next the water and the shore, what stands under a city and a tree, the rounds and corpses (items 4178, 4182, 4183); **at the pass by load** (entry 129): those landed short of the paved base and the corpse's place, each needing the simulation's door; next the base, the corpse and round, the lake's shore and floor, the figures' shadows (items 4190 to 4193) |
| **start** | a game starts from a lobby's choices with no dump of the original: map generation's remaining chain and the frame-zero start builder | the start block equal to the original's on every lobby on disk (the dumps print it) | `rondata::diff` on the start blocks | **open, 0 of 161**: the harness's lobby agrees on 9,089 of 18,264 start fields and parts at the first map record, since nothing yet composes the map chain into a `World` (item 4047); one lobby closes first, as the milestone's step 1, and the other 160 are filler behind it; **1 of 161 since item 4063**, all 18,264 fields of the harness lobby; item 4034 takes the rest under the `start` brief kind (entry 128 §7); **143 of 161** since items 4034 and 4186, the 18 left on fourteen styles with no composed chain, Southwest Mesa first (item 4196) |
| **checksum** | the original's full-state hash, ported, compared per frame against the original's own | ~~frames whose hash agrees, of the frames the original prints under `CHECKSUM=2`~~ the state lines that agree, of twelve, on the frames a `DUMP_ALL` window of the playback prints (entry 119: that setting prints no frame) | the diff suite | **open**, the milestone's step 5; the territory's group G; **at the pass by load** (entry 129 §5): eight of twelve lines on frame 91, the leaders line the last that counts, and step P takes it to three mid-raid packets (item 4188) |
| **orders** | a mouse and keyboard mint the same order stream the replays carry; the stream is the replay and, later, the network format | orders issued through the public `Command` API replaying in lockstep | the diff suite on a minted stream | **the door exists** (`sim::command`, item 4017: what the replays issue today, as forwards); minting a stream from it is the milestone's step 4; the territory's group B is its specification; **step 4 closed** (items 4059, 4149: three clicked orders played by the original for 24,000 frames); next a right press on an object and a dragged box (items 4179, 4180); both landed, each played by the original for 300 frames; next one unit's main-view fields and the attack played (items 4194, 4195) |
| **lobby** | our own screen producing the start's choices | a page of client code; no counter | the client's tests | **not started**; theirs is cut (the `iface*` windows, GameSpy); item 4184 is a plain page with a counter, the lobbies on disk whose choices it reproduces, N of 161 (entry 128 §6); **161 of 161** since item 4184; next what a person sees once the game is on, a plain HUD (item 4197, entry 129 §7) |
| **AI** (phase 5) | a computer opponent plays a whole game against a person in our client | the held-out battery, games nobody tuned for scored on their first try; then the original replaying games played here against the computer | the diff suite; the original on the capture lane | **ported as far as the captures reach**: the computer's decisions are computed here, not replayed, in every lockstep game on the scoreboard, at four difficulties; held-out 8 of 9 whole. What remains is the branches no capture entered, which a person's play will enter and the milestone's oracle will score |
| **save and replay** | a game saved as seed plus orders, replayable here and written back into the original's recorded-game container | a saved game replays to the same hash; the original replays one of ours | the diff suite; the original on the capture lane | **the container is read and the original plays a file on the click-free lane** (`docs/RECGAME.md`, the preflight); the writer is the milestone's step 3 |
| **census** | the territory count tells the truth about the original's 3,611 simulation functions | ported · replaced by design · open, the readers' marks landed row by row (`docs/audit/2026-11-06-territory-*.md`; lore's scoring of the `replaced` groups) | `tools/census.py`'s pins | filler for an idle lane; the open count reads 1,286 and the readers' marks would take it near 156 |

**The order of play** (the seventy-sixth pass, entry 116). The milestone
first, one lobby deep, on the driver's own builders; the renderer beside it
on one lane, taking textures and team colours next; then the lobby screen
and the client's input, which the milestone's steps 3 and 4 make
checkable; then breadth — the other lobbies' start blocks, M on more
lobbies, the census filler — as filler for an idle worker. The rules track
is whole and stays whole: its floors run on every gate forever.

**The workers** (entries 116, 126, 128 §4, 129 §1): the driver's builders on
the chain and three lanes, four in all, confirmed at the pass by load. At
most one lane holds an item the queue marks whole, and a lane's whole gate
runs one at a time host-wide. The count is Ramon's, and a pass moves it
only by recommending.
