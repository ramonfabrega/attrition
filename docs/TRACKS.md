# The tracks

The finish line, decomposed into the avenues the loop can run, each with
the number a landing moves and the gate that says the move was real. The
steering pass maintains this page (`docs/LOOP.md`, "the passes"); the
queue books from it; `docs/PARKED.md` is the backlog under it. A fresh
session reads the queue first and this page second, to know which track
its item serves. Written at the seventy-fourth pass, 2026-11-08, with Ramon
on the steer row (`docs/DECISIONS.md` entry 114).

## The finish line (decided with Ramon, the seventy-fourth pass)

Our own app, no Wine. It opens in a moment. A lobby asks for a map, a
nation and computer opponents; one click, and the game is on screen fast,
drawn with the original's art, running smooth and light. You play a whole
game against the computer. Under the picture the rules are the original's
exactly: fed the same orders, Rise of Nations does the same thing frame for
frame. A game can be saved and replayed, by us and by the original.

Not in it, by design: multiplayer, Conquer the World, the editor, new art.
Each is a finish line of its own, after this one is finished and tested.

## The tracks

| track | what "done" is | the number | its gate | state at the seventy-fourth pass |
|---|---|---|---|---|
| **rules** (phase 3) | every captured game replays in lockstep for its full length | the scoreboard's words, `rondata::diff`'s floors; entry 29's three counters | the release diff suite | **whole**: every word stands at its length but chapter 134 (w969 of 1,400); supply three shelved items and the parked chapters; filler from here |
| **structure** | the sim crate has a public surface and subsystem boundaries, the tests and sweeps live outside the source, nothing is defined twice | `docs/STRUCTURE.md`'s shape counts (files in the flat root, lines of test code in `src/`, duplicated names, the `Sim` struct's fields), each falling to its target; **every floor unchanged on every landing** | the whole release gate, no behaviour diff | **open**, booked from `docs/STRUCTURE.md`; runs alone on the sim side until it lands |
| **renderer** (phase 4) | a captured game drawn with the original's art, placed where the original would draw it | pieces in register K of M (`docs/RENDERER.md` §5.4); frames in register as its floor; the clock: seconds to lobby, seconds to frame zero, slowest-percentile frame milliseconds, peak resident memory | a `render` crate's pins, parsed as the queue's | **open** on Ramon's word; the pure camera and the first decoder first (§6 items 2 and 4), the scene after the structure's snapshot lands (item 1), then the draw list and first pixels |
| **start** | a game starts from a lobby's choices with no dump of the original: map generation's remaining chain and the frame-zero start builder | the start block equal to the original's on every lobby on disk (the dumps print it) | `rondata::diff` on the start blocks | **open**, the sim lane's word after the structure; the territory's group E |
| **checksum** | the original's full-state hash, ported, compared per frame against the original's own | frames whose hash agrees, of the frames the original prints under `CHECKSUM=2` | the diff suite | **open**, the sim lane's word after the structure; the territory's group G |
| **orders** | a mouse and keyboard mint the same order stream the replays carry; the stream is the replay and, later, the network format | orders issued through the public `Command` API replaying in lockstep | the diff suite on a minted stream | **not started**; the territory's group B is its specification |
| **lobby** | our own screen producing the start's choices | a page of client code; no counter | the client's tests | **not started**; theirs is cut (the `iface*` windows, GameSpy) |
| **AI** (phase 5) | a computer opponent plays a whole game | to be invented when the track opens; the scripts under `game/ai/scripts/` are open, combat and targeting are in the executable | — | **not started** |
| **save and replay** | a game saved as seed plus orders, replayable here and written back into the original's recorded-game container | a saved game replays to the same hash; the original replays one of ours | the diff suite; the original on the capture lane | **not started**; the container is read already (`docs/RECGAME.md`) |
| **census** | the territory count tells the truth about the original's 3,611 simulation functions | ported · replaced by design · open, the readers' marks landed row by row (`docs/audit/2026-11-06-territory-*.md`; lore's scoring of the `replaced` groups) | `tools/census.py`'s pins | filler for an idle lane; the open count reads 1,286 and the readers' marks would take it near 156 |

**The order of play.** Structure first, alone on the sim side, with the
renderer's two sim-free items beside it; then the start and the checksum on
the sim lane while the renderer reaches first pixels; then orders, the
lobby and save; then the AI. The rules track is whole and stays whole — its
floors run on every gate forever.

**Two lanes** (entry 114 §3), and the count moves only when a pass names
the bottleneck a third would relieve.
