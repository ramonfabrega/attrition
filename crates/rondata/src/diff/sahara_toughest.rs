//! **The closed map in the newest pair's lobby** (DECISIONS 56 §1, item
//! 1221): Great Sahara, `MAP_STYLE 7`, seed 12345, at the second pair's
//! difficulty, Toughest — run382's and run383's game with the one setting
//! the first pair's lobby held fixed moved. Three captures: run468, a
//! `DUMP_ALL` start in run381's shape at `DIFFICULTY 5`, which is the
//! sibling everything here stands up from; run469, 1,850 blocks in run33's
//! shape; and run470, the draw stream to 24,000 or the game's end at
//! `cover=0`. The word is [`THIRD_WORD_GREAT_SAHARA_TOUGHEST`], the
//! `AI_WORDS` row `GreatSaharaToughest` on the `Third map` line. No
//! mechanism is named here.

use super::*;

use crate::diff::testkit::*;
use crate::diff::third::walk_sahara_from;
use crate::testenv::{dump, install};

/// run468: the `DUMP_ALL` start at Toughest — the height table, the cells,
/// the checksum trace, the herds and the frame seeds.
pub(crate) const TOUGHEST_START: &str = "gamelog-run468-greatsahara-toughest-start.txt";
/// run469: 1,850 blocks at run10's detail, run382's shape at Toughest.
pub(crate) const TOUGHEST_SCORE: (&str, &str) = (
    "gamelog-run469-greatsahara-toughest-longtrace.txt",
    "rontrace-run469.log",
);
/// run470: the draw stream at `cover=0`, run383's shape at Toughest.
pub(crate) const TOUGHEST_LONG: (&str, &str) = (
    "gamelog-run470-greatsahara-toughest-24k-trace.txt",
    "rontrace-run470.log",
);

/// run471: run470's game at run449's detail over blocks 5371..5627, the
/// first word 5376's widening (item 1221) — six blocks before the word's
/// block 5377 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_5376: &str = "gamelog-run471-greatsahara-toughest-5376.txt";

/// **run471's window** (item 1221): blocks 5371..5627.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST: (i64, i64) = (5_371, 5_627);

/// **The first word's block, 5377**: frame 5376 writes it. The word has
/// moved on (item 1241); run471 keeps its value diff, the Granary's place.
pub(crate) const TOUGHEST_WORD_BLOCK: i64 = 5_377;

/// run476: run470's game at run471's detail over blocks 5777..6033, the
/// word 5782's widening (item 1241) — six blocks before the word's block
/// 5783 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_5782: &str = "gamelog-run476-greatsahara-toughest-5782.txt";

/// **run476's window** (item 1241): blocks 5777..6033.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_5782: (i64, i64) = (5_777, 6_033);

/// **The word's block, 5783**: frame 5782 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_5783: i64 = 5_783;

/// run483: run470's game at run476's detail over blocks 7065..7321, the
/// word 7070's widening (item 1251) — six blocks before the word's block
/// 7071 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_7070: &str = "gamelog-run483-greatsahara-toughest-7070.txt";

/// **run483's window** (item 1251): blocks 7065..7321.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_7070: (i64, i64) = (7_065, 7_321);

/// **The word's block, 7071**: frame 7070 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_7071: i64 = 7_071;

/// run488: run470's game at run483's detail over blocks 7780..8036, the
/// word 7785's widening (item 1260) — six blocks before the word's block
/// 7786 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_7785: &str = "gamelog-run488-greatsahara-toughest-7785.txt";

/// **run488's window** (item 1260): blocks 7780..8036.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_7785: (i64, i64) = (7_780, 8_036);

/// **The word's block, 7786**: frame 7785 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_7786: i64 = 7_786;

/// run491: run470's game at run488's detail over blocks 8177..8433, the
/// word 8182's widening (item 1264) — six blocks before the word's block
/// 8183 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_8182: &str = "gamelog-run491-greatsahara-toughest-8182.txt";

/// **run491's window** (item 1264): blocks 8177..8433.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_8182: (i64, i64) = (8_177, 8_433);

/// **The word's block, 8183**: frame 8182 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_8183: i64 = 8_183;

/// **The word's block, 8378**: frame 8377 writes it — the word item 1275
/// left, inside run491's window.
pub(crate) const TOUGHEST_WORD_BLOCK_8378: i64 = 8_378;

/// run500: run470's game at run491's detail over blocks 8781..9037, the
/// word 8786's widening (item 1286) — six blocks before the word's block
/// 8787 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_8786: &str = "gamelog-run500-greatsahara-toughest-8786.txt";

/// **run500's window** (item 1286): blocks 8781..9037.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_8786: (i64, i64) = (8_781, 9_037);

/// **The word's block, 8787**: frame 8786 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_8787: i64 = 8_787;

/// **The word's block, 8857**: frame 8856 writes it — the word item 1293
/// left, inside run500's window.
pub(crate) const TOUGHEST_WORD_BLOCK_8857: i64 = 8_857;

/// run511: run470's game at run500's detail over blocks 9318..9574, the
/// word 9323's widening (item 1305) — six blocks before the word's block
/// 9324 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_9323: &str = "gamelog-run511-greatsahara-toughest-9323.txt";

/// **run511's window** (item 1305): blocks 9318..9574.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_9323: (i64, i64) = (9_318, 9_574);

/// **The word's block, 9324**: frame 9323 writes it — item 1318's value
/// diff, and the coverage driver's window from item 1305.
pub(crate) const TOUGHEST_WORD_BLOCK_9324: i64 = 9_324;

/// **The word's block, 9353**: frame 9352 writes it — the word item 1318
/// left, inside run511's window.
pub(crate) const TOUGHEST_WORD_BLOCK_9353: i64 = 9_353;

/// run529: run470's game at run500's detail over blocks 9759..10015, the
/// word 9764's widening (item 1332) — six blocks before the word's block
/// 9765 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_9764: &str = "gamelog-run529-greatsahara-toughest-9764.txt";

/// **run529's window** (item 1332): blocks 9759..10015.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_9764: (i64, i64) = (9_759, 10_015);

/// **The word's block, 9765**: frame 9764 writes it — the word item 1332
/// left, past run511's window.
pub(crate) const TOUGHEST_WORD_BLOCK_9765: i64 = 9_765;

/// **The word's block, 9983**: frame 9982 writes it — the word item 1338
/// left, inside run529's window.
pub(crate) const TOUGHEST_WORD_BLOCK_9983: i64 = 9_983;

/// **The word's block, 10000**: frame 9999 writes it — the word item 1346
/// left, inside run529's window.
pub(crate) const TOUGHEST_WORD_BLOCK_10000: i64 = 10_000;

/// run547: run470's game at run529's detail over blocks 10139..10395, the
/// word 10144's widening (item 1354) — six blocks before the word's block
/// 10145 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_10144: &str = "gamelog-run547-greatsahara-toughest-10144.txt";

/// **run547's window** (item 1354): blocks 10139..10395.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_10144: (i64, i64) = (10_139, 10_395);

/// **The word's block, 10145**: frame 10144 writes it — the word item 1354
/// left, past run529's window.
pub(crate) const TOUGHEST_WORD_BLOCK_10145: i64 = 10_145;

/// **The word's block, 10392**: frame 10391 writes it — the word item 1365
/// left, inside run547's window.
pub(crate) const TOUGHEST_WORD_BLOCK_10392: i64 = 10_392;

/// run562: run470's game at run547's detail over blocks 10774..11030, the
/// word 10779's widening (item 1371) — six blocks before the word's block
/// 10780 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_10779: &str = "gamelog-run562-greatsahara-toughest-10779.txt";

/// **run562's window** (item 1371): blocks 10774..11030.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_10779: (i64, i64) = (10_774, 11_030);

/// **The word's block, 10780**: frame 10779 writes it — the word item 1371
/// left, past run547's window.
pub(crate) const TOUGHEST_WORD_BLOCK_10780: i64 = 10_780;

/// run571: run470's game at run562's detail over blocks 11177..11433, the
/// word 11182's widening (item 1379) — six blocks before the word's block
/// 11183 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_11182: &str = "gamelog-run571-greatsahara-toughest-11182.txt";

/// **run571's window** (item 1379): blocks 11177..11433.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_11182: (i64, i64) = (11_177, 11_433);

/// **The word's block, 11183**: frame 11182 writes it — the word item 1379
/// left, past run562's window.
pub(crate) const TOUGHEST_WORD_BLOCK_11183: i64 = 11_183;

/// **The word's block, 11383**: frame 11382 writes it — the word item 1388
/// left, inside run571's window.
pub(crate) const TOUGHEST_WORD_BLOCK_11383: i64 = 11_383;

/// **The word's block, 11986**: frame 11985 writes it — the word item 1416
/// left, inside run574's window.
pub(crate) const TOUGHEST_WORD_BLOCK_11986: i64 = 11_986;

/// run574: run470's game at run571's detail over blocks 11876..12133, the
/// word 11882's widening (item 1416) — six blocks before the word's block
/// 11883 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_11882: &str = "gamelog-run574-greatsahara-toughest-11882.txt";

/// **run574's window** (item 1416): blocks 11876..12133.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_11882: (i64, i64) = (11_876, 12_133);

/// **The word's block, 11883**: frame 11882 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_11883: i64 = 11_883;

/// run584: run470's game at run574's detail over blocks 12532..12789, the
/// word 12538's widening (item 1429) — six blocks before the word's block
/// 12539 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_12538: &str = "gamelog-run584-greatsahara-toughest-12538.txt";

/// **run584's window** (item 1429): blocks 12532..12789.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_12538: (i64, i64) = (12_532, 12_789);

/// **The word's block, 12539**: frame 12538 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_12539: i64 = 12_539;

/// **The word's block, 12745**: frame 12744 writes it — the word item 1472
/// left, inside run584's window.
pub(crate) const TOUGHEST_WORD_BLOCK_12745: i64 = 12_745;

/// run640's second take: run470's game at run584's detail over the blocks
/// 12811..12834, the new word 12816's own window (item 1477). The first take
/// is the packet at logger frame 12575 (`docs/RUNS.md`).
pub(crate) const TOUGHEST_WORD_12816: &str = "gamelog-run640-greatsahara-toughest-12816.txt";

/// **run640's window** (item 1477): blocks 12811..12834.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_12816: (i64, i64) = (12_811, 12_835);

/// **The word's block, 12817**: frame 12816 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_12817: i64 = 12_817;

/// run653: run470's game at run584's detail over the blocks 14357..14380, the
/// new word 14363's own window (item 1493).
pub(crate) const TOUGHEST_WORD_14363: &str = "gamelog-run653-greatsahara-toughest-14363.txt";

/// **run653's window** (item 1493): blocks 14357..14380.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_14363: (i64, i64) = (14_357, 14_381);

/// **The word's block, 14364**: frame 14363 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_14364: i64 = 14_364;

/// run659: run470's game at run584's detail over the blocks 14506..14529, the
/// new word 14512's own window (item 1503).
pub(crate) const TOUGHEST_WORD_14512: &str = "gamelog-run659-greatsahara-toughest-14512.txt";

/// **run659's window** (item 1503): blocks 14506..14529.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_14512: (i64, i64) = (14_506, 14_530);

/// **The word's block, 14513**: frame 14512 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_14513: i64 = 14_513;

/// run663: run470's game at run584's detail over the blocks 15095..15118, the
/// new word 15101's own window (item 1510).
pub(crate) const TOUGHEST_WORD_15101: &str = "gamelog-run663-greatsahara-toughest-15101.txt";

/// **run663's window** (item 1510): blocks 15095..15118.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_15101: (i64, i64) = (15_095, 15_119);

/// **The word's block, 15102**: frame 15101 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_15102: i64 = 15_102;

/// run664: run470's game at run584's detail over the blocks 15207..15230, the
/// new word 15213's own window (item 1517).
pub(crate) const TOUGHEST_WORD_15213: &str = "gamelog-run664-greatsahara-toughest-15213.txt";

/// **run664's window** (item 1517): blocks 15207..15230.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_15213: (i64, i64) = (15_207, 15_231);

/// **The word's block, 15214**: frame 15213 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_15214: i64 = 15_214;

/// run668: run470's game at run584's detail over the blocks 15230..15433 —
/// run664's last block to the game's end, the new word 15275 inside it
/// (item 1524).
pub(crate) const TOUGHEST_WORD_15275: &str = "gamelog-run668-greatsahara-toughest-15275.txt";

/// **run668's window** (item 1524): blocks 15230..15433, the game's last.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_15275: (i64, i64) = (15_230, 15_434);

/// **The word's block, 15276**: frame 15275 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_15276: i64 = 15_276;

/// run666: run664's game with `AMMO=5` added to the detail, over the blocks
/// 15196..15230 (item 1522) — the only capture on disk that prints the
/// original's own rounds on the third map.
pub(crate) const TOUGHEST_WORD_15213_AMMO: &str = "gamelog-run666-greatsahara-toughest-ammo.txt";

/// run517: run470's game at run500's detail over blocks 9032..9323 — the
/// dark gap 9038..9317 between run500's last block and run511's first,
/// with six blocks of each either side (item 1318). The 571 keys that stood
/// on run511's first block parted in it.
pub(crate) const TOUGHEST_GAP_9038: &str = "gamelog-run517-greatsahara-toughest-9038.txt";

/// **run517's window** (item 1318): blocks 9032..9323.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_GAP: (i64, i64) = (9_032, 9_323);

/// **The road search 7070 was** (item 1260): caravan `1/52`'s replan from
/// `1/2007` to `1/2022`, which run483's trace carries node for node.
pub(crate) const TOUGHEST_ROAD_FRAME_7070: i64 = 7_070;

/// run482: run470's game at `end:MISC,LEADERS=2` over blocks 1..5378 —
/// every leader's goods record, `LeaderDataEncrypt::log_data` and the
/// gather-slot arrays beside it, on every frame the dark gap between run469
/// and run471 left unprinted (item 1251).
pub(crate) const TOUGHEST_GOODS: &str = "gamelog-run482-greatsahara-toughest-goods.txt";

/// run495: run470's game at run482's `end:MISC,LEADERS=2` over blocks
/// 6030..7066 — every leader's goods through the gap between run476's last
/// block and run483's first, where who=1's wealth `leftover` parted
/// (item 1275).
pub(crate) const TOUGHEST_WEALTH: &str = "gamelog-run495-greatsahara-toughest-wealth.txt";

/// **run495's window** (item 1275): blocks 6030..7066.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_WEALTH: (i64, i64) = (6_030, 7_066);

/// **run482's window** (item 1251): blocks 1..5378.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_GOODS: (i64, i64) = (1, 5_378);

/// **The block the goods first parted on, 4577**: frame 4576, the script
/// step that researches who=1's Empire (item 1251).
pub(crate) const TOUGHEST_GOODS_BLOCK_4577: i64 = 4_577;

/// run471's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST`], walked from
/// run470's start with run468's head, as `third::sahara_17623_window` walks
/// run449: every record run449's detail prints, the group record and the
/// attack row's pass included. `None` when the captures are not on this
/// machine.
pub(crate) fn great_sahara_toughest_5376_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run471",
        (TOUGHEST_WORD_5376, WIDENING_GREAT_SAHARA_TOUGHEST.0),
        WIDENING_GREAT_SAHARA_TOUGHEST,
        TOUGHEST_WORD_BLOCK,
    )
}

/// run476's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_5782`], walked as
/// run471's are (item 1241).
pub(crate) fn great_sahara_toughest_5782_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run476",
        (TOUGHEST_WORD_5782, WIDENING_GREAT_SAHARA_TOUGHEST_5782.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_5782,
        TOUGHEST_WORD_BLOCK_5783,
    )
}

/// run483's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_7070`], walked as
/// run476's are (item 1251).
pub(crate) fn great_sahara_toughest_7070_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run483",
        (TOUGHEST_WORD_7070, WIDENING_GREAT_SAHARA_TOUGHEST_7070.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_7070,
        TOUGHEST_WORD_BLOCK_7071,
    )
}

/// run488's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_7785`], walked as
/// run483's are (item 1260).
pub(crate) fn great_sahara_toughest_7785_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run488",
        (TOUGHEST_WORD_7785, WIDENING_GREAT_SAHARA_TOUGHEST_7785.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_7785,
        TOUGHEST_WORD_BLOCK_7786,
    )
}

/// run482's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_GOODS`], walked as
/// run471's are (item 1251): the goods of every leader from block 1.
pub(crate) fn great_sahara_toughest_goods_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run482",
        (TOUGHEST_GOODS, WIDENING_GREAT_SAHARA_TOUGHEST_GOODS.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_GOODS,
        TOUGHEST_GOODS_BLOCK_4577,
    )
}

/// run495's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_WEALTH`], walked as
/// run482's are (item 1275): the goods of every leader through the gap.
pub(crate) fn great_sahara_toughest_wealth_window() -> Option<crate::diff::harness::tests::Widened>
{
    toughest_window_over(
        "run495",
        (TOUGHEST_WEALTH, WIDENING_GREAT_SAHARA_TOUGHEST_WEALTH.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_WEALTH,
        TOUGHEST_WORD_BLOCK_7071,
    )
}

/// run491's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_8182`], walked as
/// run488's are (item 1264).
pub(crate) fn great_sahara_toughest_8182_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run491",
        (TOUGHEST_WORD_8182, WIDENING_GREAT_SAHARA_TOUGHEST_8182.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_8182,
        TOUGHEST_WORD_BLOCK_8183,
    )
}

/// run500's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_8786`], walked as
/// run491's are (item 1286).
pub(crate) fn great_sahara_toughest_8786_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run500",
        (TOUGHEST_WORD_8786, WIDENING_GREAT_SAHARA_TOUGHEST_8786.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_8786,
        TOUGHEST_WORD_BLOCK_8787,
    )
}

/// run511's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_9323`], walked as
/// run500's are (item 1305).
pub(crate) fn great_sahara_toughest_9323_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run511",
        (TOUGHEST_WORD_9323, WIDENING_GREAT_SAHARA_TOUGHEST_9323.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_9323,
        TOUGHEST_WORD_BLOCK_9353,
    )
}

/// run529's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_9764`], walked as
/// run511's are (item 1332).
pub(crate) fn great_sahara_toughest_9764_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run529",
        (TOUGHEST_WORD_9764, WIDENING_GREAT_SAHARA_TOUGHEST_9764.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_9764,
        TOUGHEST_WORD_BLOCK_9765,
    )
}

/// run547's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_10144`], walked as
/// run529's are (item 1354).
pub(crate) fn great_sahara_toughest_10144_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run547",
        (TOUGHEST_WORD_10144, WIDENING_GREAT_SAHARA_TOUGHEST_10144.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_10144,
        TOUGHEST_WORD_BLOCK_10145,
    )
}

/// run562's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_10779`], walked as
/// run547's are (item 1371).
pub(crate) fn great_sahara_toughest_10779_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run562",
        (TOUGHEST_WORD_10779, WIDENING_GREAT_SAHARA_TOUGHEST_10779.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_10779,
        TOUGHEST_WORD_BLOCK_10780,
    )
}

/// run571's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_11182`], walked as
/// run562's are (item 1379).
pub(crate) fn great_sahara_toughest_11182_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run571",
        (TOUGHEST_WORD_11182, WIDENING_GREAT_SAHARA_TOUGHEST_11182.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_11182,
        TOUGHEST_WORD_BLOCK_11183,
    )
}

/// run574's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_11882`], walked as
/// run571's are (item 1416).
pub(crate) fn great_sahara_toughest_11882_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run574",
        (TOUGHEST_WORD_11882, WIDENING_GREAT_SAHARA_TOUGHEST_11882.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_11882,
        TOUGHEST_WORD_BLOCK_11883,
    )
}

/// run584's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_12538`], walked as
/// run574's are (item 1429).
pub(crate) fn great_sahara_toughest_12538_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run584",
        (TOUGHEST_WORD_12538, WIDENING_GREAT_SAHARA_TOUGHEST_12538.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_12538,
        TOUGHEST_WORD_BLOCK_12539,
    )
}

/// run640's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_12816`], walked as
/// run584's are (item 1477).
pub(crate) fn great_sahara_toughest_12816_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run640",
        (TOUGHEST_WORD_12816, WIDENING_GREAT_SAHARA_TOUGHEST_12816.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_12816,
        TOUGHEST_WORD_BLOCK_12817,
    )
}

/// run653's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_14363`], walked as
/// run584's are (item 1493).
pub(crate) fn great_sahara_toughest_14363_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run653",
        (TOUGHEST_WORD_14363, WIDENING_GREAT_SAHARA_TOUGHEST_14363.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_14363,
        TOUGHEST_WORD_BLOCK_14364,
    )
}

/// run517's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_GAP`], walked as
/// run511's are (item 1318).
pub(crate) fn great_sahara_toughest_gap_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run517",
        (TOUGHEST_GAP_9038, WIDENING_GREAT_SAHARA_TOUGHEST_GAP.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_GAP,
        TOUGHEST_WORD_BLOCK_9324,
    )
}

/// run659's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_14512`], walked as
/// run584's are (item 1503).
pub(crate) fn great_sahara_toughest_14512_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run659",
        (TOUGHEST_WORD_14512, WIDENING_GREAT_SAHARA_TOUGHEST_14512.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_14512,
        TOUGHEST_WORD_BLOCK_14513,
    )
}

/// run663's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_15101`], walked as
/// run584's are (item 1510).
pub(crate) fn great_sahara_toughest_15101_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run663",
        (TOUGHEST_WORD_15101, WIDENING_GREAT_SAHARA_TOUGHEST_15101.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_15101,
        TOUGHEST_WORD_BLOCK_15102,
    )
}

/// run664's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_15213`], walked as
/// run584's are (item 1517).
pub(crate) fn great_sahara_toughest_15213_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run664",
        (TOUGHEST_WORD_15213, WIDENING_GREAT_SAHARA_TOUGHEST_15213.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_15213,
        TOUGHEST_WORD_BLOCK_15214,
    )
}

/// run668's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_15275`], walked as
/// run584's are (item 1524).
pub(crate) fn great_sahara_toughest_15275_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run668",
        (TOUGHEST_WORD_15275, WIDENING_GREAT_SAHARA_TOUGHEST_15275.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_15275,
        TOUGHEST_WORD_BLOCK_15276,
    )
}

/// **The compared pin's window on the third map at Toughest** (item 1221):
/// the open word [`THIRD_WORD_GREAT_SAHARA_TOUGHEST`]'s block and two on
/// either side, as `second::east_indies_word_window` takes East Indies',
/// on run664 at its block 15214 since item 1517 (run663 at its block 15102
/// from item 1510, run659 at its block 14513
/// from item 1503, run653 at its block 14364
/// from item 1493, run640 at its block 12817
/// from item 1477, run584 at its block 12745
/// from item 1472, at its block 12570 from
/// item 1429; run574 at its block 11986
/// from item 1416, run571 at its block 11183
/// from item 1379, run562 at its block 10780
/// from item 1371, run547 at its block 10392
/// from item 1365, at its block 10145 from
/// item 1354, run529 at its block 9765
/// from item 1332, run511 from item 1305,
/// at its block 9353 from item 1318; run500 from item 1286, at
/// its block 8857 from item 1293; run491
/// from item 1264, at its block 8378 from item 1275; run488 from item 1260, run483 from item 1251, run476
/// from item 1241, run471 before it). `coverage`'s compared pin walks these
/// blocks with the recorder on.
pub(crate) fn great_sahara_toughest_word_window() -> Option<crate::diff::harness::tests::Widened> {
    // Frame `f` writes block `f + 1`, and the walk reads `first..=tail`.
    let word = TOUGHEST_WORD_BLOCK_15214 - 1;
    toughest_window_over(
        "run664",
        (TOUGHEST_WORD_15213, WIDENING_GREAT_SAHARA_TOUGHEST_15213.0),
        (word - 1, word + 2),
        TOUGHEST_WORD_BLOCK_15214,
    )
}

fn toughest_window_over(
    run: &str,
    capture: (&str, i64),
    window: (i64, i64),
    word_block: i64,
) -> Option<crate::diff::harness::tests::Widened> {
    crate::diff::harness::tests::widen_on_siblings(
        &[TOUGHEST_START],
        true,
        TOUGHEST_LONG,
        run,
        &[capture],
        window,
        1,
        &[word_block],
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **run468 and run469 are Great Sahara at Toughest**: one `GAME INFO`
    /// between them, and it is run381's but for `DIFFICULTY 5` against 0 —
    /// the one setting moved (DECISIONS 53 §2, 56 §1).
    #[test]
    fn run469_is_great_sahara_at_toughest_and_nothing_else_moved() {
        let (Some(start), Some(score), Some(first)) = (
            dump(TOUGHEST_START),
            dump(TOUGHEST_SCORE.0),
            dump(crate::diff::third::SAHARA_START),
        ) else {
            eprintln!("skipping: no run468/run469/run381 (set RON_GAMELOG_DIR)");
            return;
        };
        let (start_text, score_text, first_text) = (
            crate::capture::read(&start),
            crate::capture::read(&score),
            crate::capture::read(&first),
        );
        let (start_log, score_log, first_log) = (
            Log::parse(&start_text),
            Log::parse(&score_text),
            Log::parse(&first_text),
        );
        let start = start_log.initial().expect("run468 is a start dump");
        let score = score_log.initial().expect("run469's head");
        let first = first_log.initial().expect("run381 is a start dump");
        assert!(
            !start.heights.is_empty() && !start.herds.is_empty(),
            "run468 carries the height table and the herds"
        );
        assert!(
            !start.game_info.is_empty() && start.game_info == score.game_info,
            "run468 and run469 print one GAME INFO"
        );
        assert_eq!(start.game_info.len(), first.game_info.len());
        let apart: Vec<_> = start
            .game_info
            .iter()
            .zip(first.game_info.iter())
            .filter(|(a, b)| a != b)
            .collect();
        assert_eq!(
            apart,
            [(&("DIFFICULTY", "5"), &("DIFFICULTY", "0"))],
            "run468's GAME INFO against run381's"
        );
    }

    /// **The closed map's score at Toughest** (item 1221): run469 walked
    /// from its own head with run468's borrowed. Measured, and held where
    /// it stands: 1850 and 1850, run382's to the frame — the one unit that
    /// parts is the human citizen `0/5` on the shutdown block 1851.
    #[test]
    fn run469_s_score_holds() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump(TOUGHEST_SCORE.0),
            dump(TOUGHEST_START),
            trace(TOUGHEST_SCORE.1),
        ) else {
            eprintln!("skipping: no run469/run468 (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run468 is a start dump");
        let report = run_traced(
            &loaded,
            &log,
            Tuning::RON,
            None,
            None,
            &[&sib_init],
            Some(&tr),
        )
        .unwrap();
        assert_eq!(report.frames.len(), 1851, "run469's length");
        assert!(
            report
                .notes
                .iter()
                .any(|n| n.starts_with("rng: frame 0: ours 100 draws, the original's 100")),
            "frame 0 at Toughest: {:?}",
            report.notes
        );
        let ticks = report.ticks_before_divergence();
        let orders = report.order_ticks_before_divergence();
        assert!(
            ticks >= 1850 && orders >= 1850,
            "run469's score fell: ticks {ticks}, orders {orders}, first divergence {:?}",
            report.first_divergence
        );
    }

    /// **The word on the 1,850-frame capture**: run469's trace, frame for
    /// frame, against this crate's draws, walked from run468's start.
    #[test]
    fn run469_s_trace_holds_to_its_end() {
        let Some(w) = walk_sahara_from(TOUGHEST_START, TOUGHEST_SCORE) else {
            return;
        };
        assert_eq!(w.difficulty, 5, "run469's GAME INFO reads DIFFICULTY 5");
        assert!(
            w.count >= w.last && w.sequence >= w.last,
            "run469's draws part: count {}, sequence {} of {}; {}",
            w.count,
            w.sequence,
            w.last,
            w.row
        );
    }

    /// **The third map's first word at Toughest, 5376, widened whole** (item
    /// 1221): [`great_sahara_toughest_5376_window`] over run471's blocks
    /// 5371..5627, both directions, every record run449's detail prints.
    /// The word's frame writes block 5377. Item 1241 moved the word past
    /// it: `find_friends`' enhancer arm puts the Granary where the
    /// original does (`docs/AI.md` §99.7).
    #[test]
    fn run471_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_5376_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 257, "run471 whole: blocks 5371..5627");
        pin!(
            w.missing.is_empty(),
            "run471 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 5371 stands on 85 keys**, the families the first pair's
        // widenings carry: `form` on every citizen and soldier, the human's
        // census and city rows (this crate fills neither for player 0),
        // both leaders' `SITE[i].reg` (ours 1, theirs 0), who=1's army
        // groups' `role` and pool lists, and two rows of who=1's own that
        // this crate reads: `scholars` 0 against 1 and the
        // `tech_cat_frame`s. Who=1's `bucket[0:food]` stood too, 381 against
        // 417, until item 1251 priced frame 4576's Empire as the original
        // does (run482; `docs/AI.md` §99.8).
        pin_eq!(
            row(1, -1, "leader:bucket[0:food]"),
            None,
            "who=1's food bucket agrees"
        );
        // **Frame 5375's block adds who=1's `SITE[2].reg`**, and the word's
        // block 5377 its 42: the value diff below.
        pin_eq!(
            row(1, -1, "leader:SITE[2].reg").as_deref(),
            Some("5376: ours 1 theirs 0"),
            "who=1's third site's region"
        );
        // Item 1457: [(5371, 60), (5376, 1)] → [(5371, 56), (5376, 1)]; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= TOUGHEST_WORD_BLOCK)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(5371, 49), (5376, 1)],
            "the blocks keys first part on, to the first word's"
        );
        // **The first word's value diff, block 5377, closed by item 1241**:
        // who=1's Granary `1/2023` stood at ours (40608, 19680) against the
        // original's (41184, 15072), and the two citizens sent to it parted
        // with it (`1/12`'s order kind 3 against 7, `1/8`'s 1 against 3).
        // With the enhancer arm the two farms north of the city count as
        // its friends at cell (53, 19), and all four rows agree.
        for (o, what) in [
            (2023, "build:x_internal"),
            (2023, "build:y_internal"),
            (12, "order:kind"),
            (8, "order:kind"),
        ] {
            pin_eq!(row(1, o, what), None, "1/{o}'s {what} agrees on 5377");
        }
        // **The make list's values agree** since item 1251: on 5380 who=1's
        // `MAKE[2]` and `MAKE[3]` swapped and `MAKE[9].val` read 4800
        // against 48000 — Feudalism, a civic epoch, unaffordable at the
        // price without Dye's quarter. What stands of the list is its
        // `city`, ours one over the original's from 5382.
        pin_eq!(
            row(1, -1, "leader:MAKE[9].val"),
            None,
            "who=1's tenth make row agrees"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was Some("5382: ours 1 theirs 0").
        pin_eq!(
            row(1, -1, "leader:MAKE[1].city").as_deref(),
            None,
            "the make list's city stands"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 202.
        // Item 1354, the `mylos` cache (`docs/VISION.md` §2): 167 → 144, the
        // 23 Citizens' `mylos` of block 5544, ours 4 against 2 a frame
        // ahead of the original's refresh.
        // Item 1457: 140 → 136; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
        pin_eq!(w.firsts.len(), 123, "every key parted on run471");
    }

    /// **The third map's word at Toughest, 5782, widened whole** (item
    /// 1241): [`great_sahara_toughest_5782_window`] over run476's blocks
    /// 5777..6033, both directions, every record run471's detail prints.
    /// The word's frame writes block 5783. Item 1251 named its cause, 36
    /// food short from frame 4576, and moved the word to 7070.
    #[test]
    fn run476_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_5782_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 257, "run476 whole: blocks 5777..6033");
        pin!(
            w.missing.is_empty(),
            "run476 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 5777 stands on 102 keys**: run471's families — `form`,
        // the human's census rows, both leaders' `SITE[i].reg` (ours 1,
        // theirs 0). Who=1's food bucket stood too, 90 against 126, until
        // item 1251 priced frame 4576's Empire with Dye's quarter off
        // (`docs/AI.md` §99.8); it agrees.
        pin_eq!(
            row(1, -1, "leader:bucket[0:food]"),
            None,
            "who=1's food bucket agrees"
        );
        // **The make list's values agree**: `MAKE[2].val` and `MAKE[9].val`
        // read 1200 against 4800 on 5779 before item 1251 — the Feudalism
        // offer, a civic epoch the food could not reach at our price. On
        // 5781 six rows' `city` part, ours one over the original's.
        pin_eq!(
            row(1, -1, "leader:MAKE[9].val"),
            None,
            "who=1's tenth make row agrees"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was [(5777, 102), (5781, 6)].
        // Item 1457: [(5777, 71)] → [(5777, 67)]; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= TOUGHEST_WORD_BLOCK_5783)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(5777, 52)],
            "the blocks keys first part on, to the old word's"
        );
        // **The old word's value diff, block 5783, closed by item 1251**:
        // ours spent a `Leader::make_stuff+0x63d` the original did not,
        // queuing one more Hoplite at `1/2017` — `queued` 2 against 1,
        // `num_queued[82]` 1 against 0 — and paying for it: metal 7
        // against 43. The original's market had sold the food that would
        // have paid; with the 36 back all three agree.
        for (o, what) in [
            (2017, "queue:queued"),
            (-1, "leader:num_queued[82]"),
            (-1, "leader:bucket[4:metal]"),
        ] {
            pin_eq!(row(1, o, what), None, "1/{o}'s {what} agrees on 5783");
        }
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 147.
        // Item 1377: 103 → 102, who=1's `defense` (`Build::init`'s `+1`).
        // Item 1457: 98 → 88; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
        pin_eq!(w.firsts.len(), 60, "every key parted on run476");
    }

    /// **The third map's word at Toughest, 7070, widened whole** (item
    /// 1251): [`great_sahara_toughest_7070_window`] over run483's blocks
    /// 7065..7321, both directions, every record run476's detail prints.
    /// The word's frame writes block 7071. No mechanism is named.
    #[test]
    fn run483_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_7070_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 257, "run483 whole: blocks 7065..7321");
        pin!(
            w.missing.is_empty(),
            "run483 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 7065 stands on 127 keys** (129 before item 1275): run476's families — `form`,
        // the human's census and city rows, `SITE[i].reg`, the make list's
        // `city` one over — and three of who=1's that part in the gap past
        // run476: wealth 161 against 162, its `leftover` 1584 against 48,
        // and `MAKE[4].val` 1431372 against 1228956.
        // **The wealth rows agree since item 1275**: they parted on
        // run495's 6592, when the original's `do_trade` re-summed city 2's
        // delivered route for `1/52`'s new one (`docs/AI.md` §99.10).
        pin_eq!(
            row(1, -1, "leader:bucket[2:wealth]"),
            None,
            "who=1's wealth agrees"
        );
        pin_eq!(
            row(1, -1, "leader:leftover[2:wealth]"),
            None,
            "who=1's wealth leftover agrees"
        );
        // Item 1451, `largest_gather` (AI §115): MAKE[4].val 1431372/1228956 agrees.
        pin_eq!(
            row(1, -1, "leader:MAKE[4].val").as_deref(),
            None,
            "who=1's fifth make row stands"
        );
        // **The word's block 7071 is quiet**: nothing parts between the
        // standing block and 7075. Frame 7070's extra draws are one road
        // search, `1/52`'s — ours 3204 `PathFinder::calc_road_cost+0x46`
        // against the original's 2543 — and no record prints a search.
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was [(7065, 119)].
        // Item 1451, `largest_gather` (AI §115): 71 → 70.
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= TOUGHEST_WORD_BLOCK_7071 + 3)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(7065, 45)],
            "the blocks keys first part on, to three past the word's"
        );
        // **The word moved to 7785 on item 1260**, past this window: with
        // `leech_codes` the road beside the Farm stands and 7070's search
        // arrives as the original's does. What parts past the standing
        // block is five keys — who=1's `peasants` 30 against 31 and `1/56`'s
        // `form` on 7144 first — where it was 616.
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        // Item 1451, `largest_gather` (AI §115): 71 → 70.
        pin_eq!(
            by.iter().map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(7065, 45), (7201, 1)],
            "the blocks keys first part on, the window whole"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 124.
        // Item 1451, `largest_gather` (AI §115): 72 → 71, MAKE[4].val.
        pin_eq!(
            w.firsts.len(),
            46,
            "every key parted on run483 (132 before item 1281, 134 before item 1275, 745 before item 1260)"
        );
    }

    /// **The third map's word at Toughest, 7785, widened whole** (item
    /// 1260): [`great_sahara_toughest_7785_window`] over run488's blocks
    /// 7780..8036, both directions, every record run483's detail prints.
    /// The word's frame writes block 7786. No mechanism is named.
    #[test]
    fn run488_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_7785_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 257, "run488 whole: blocks 7780..8036");
        pin!(
            w.missing.is_empty(),
            "run488 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 7780 stands on 130 keys** (132 before item 1275's trade):
        // run483's families, and new past it who=1's `known_rares`, ours one
        // short.
        pin_eq!(
            row(1, -1, "leader:known_rares").as_deref(),
            Some("7780: ours 3 theirs 4"),
            "who=1's known rares stand"
        );
        // **The make list parted before the word** (item 1251's 7785): on
        // 7785 its third row was a Senate (438) in category 8 here and a
        // Mine (419) in 4 there, and on 7786 both laid the Senate `1/2030`
        // apart. **Item 1264 took both** (`docs/TECH.md` step 8): who=1's
        // Tower is a Keep in the original's `num_buildings` from before
        // run483's 7065 (Tower 1, Keep 0 through run476's 6033), and this
        // crate kept a Tower. **The move's value diff:** on 7785 `MAKE[2].t`
        // ours 438 against 419 → agreeing; on 7786 `1/2030`'s
        // `x_internal` 38976 against 38784 and `y_internal` 19584 against
        // 17760 → agreeing. The word went 7785 → 8182, past this window.
        pin_eq!(
            row(1, -1, "leader:MAKE[2].t"),
            None,
            "who=1's third make row"
        );
        pin_eq!(row(1, 2030, "build:x_internal"), None, "the Senate's x");
        pin_eq!(row(1, 2030, "build:y_internal"), None, "the Senate's y");
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        // Item 1451, `largest_gather` (AI §115): 7782's two keys agree.
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= TOUGHEST_WORD_BLOCK_7786 + 3)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(7780, 48)],
            "the blocks keys first part on, to three past the word's"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 138.
        // Item 1451, `largest_gather` (AI §115): 82 → 80, MAKE[0].val and MAKE[4].val.
        pin_eq!(
            w.firsts.len(),
            51,
            "every key parted on run488 (146 before item 1281, 150 after item 1264, 876 before)"
        );
    }

    /// **The third map's word at Toughest, 8182, widened whole** (item
    /// 1264): [`great_sahara_toughest_8182_window`] over run491's blocks
    /// 8177..8433, both directions, every record run488's detail prints.
    /// The word's frame writes block 8183; the word 8377 it left (item
    /// 1275) writes 8378, inside the same window. run494 is the same
    /// capture, taken beside it (`docs/RUNS.md`).
    #[test]
    fn run491_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_8182_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 257, "run491 whole: blocks 8177..8433");
        pin!(
            w.missing.is_empty(),
            "run491 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The make list parts before the word**: on 8181 who=1's fifth
        // row is a Merchant (`TypeIndex` 61, `unitrules.xml`) at 936,170
        // here and Trade (`TypeIndex` 560, `techrules.xml`) at 307,560
        // there. It still does.
        pin_eq!(
            row(1, -1, "leader:MAKE[4].t").as_deref(),
            None,
            "who=1's fifth make row"
        );
        pin_eq!(
            row(1, -1, "leader:MAKE[4].val").as_deref(),
            None,
            "its offer"
        );
        // **The word 8182 moved** (item 1275): the Barracks `1/2017` took
        // one King's Longbowman and then the Hoplites here where the
        // original took two (`num_queued[128]` 2 against 3, `queue[2].type`
        // 132 against 178 on block 8183), its second 74 wealth paid by the
        // unit who=1's wealth `leftover` handed in on 8182 — 1398 against
        // 7062 on 8177, parted since 6592 — once `do_trade` recomputes a new
        // route's cities (`docs/AI.md` §99.10). Nothing parts on 8183 now,
        // and the leftover stands no more.
        pin_eq!(
            row(1, -1, "leader:leftover[2:wealth]"),
            None,
            "who=1's wealth leftover agrees"
        );
        // **The word 8377 moved** (item 1286): on block 8378 ours' head was
        // spent (`MAKE[0].val` −1) where the original's was a category-9
        // row worth 1981477 at (30, 24), and who=1's wealth was 172 against
        // 53 — ours bought its fourth city at 60 food and timber, counting
        // Small Cities alone, where the original priced it as the fourth
        // of the line at 160, quartered the three sites as unaffordable and
        // went to the market (`docs/AI.md` §99.11). Nothing parts on 8378
        // now, nor on any block past 8277 in the window.
        pin_eq!(
            row(1, -1, "leader:MAKE[0].val"),
            None,
            "who=1's head agrees on 8378"
        );
        pin_eq!(
            row(1, -1, "leader:bucket[2:wealth]"),
            None,
            "who=1's wealth agrees on 8378"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        // Item 1457: [(8177, 77), (8201, 1), (8209, 15), (8210, 2)] → [(8177, 77), (8201, 1), (8209, 9)]; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= TOUGHEST_WORD_BLOCK_8378 + 3)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(8177, 48), (8201, 1)],
            "the blocks keys first part on, to three past the word 8377's"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 157.
        pin_eq!(
            w.firsts.len(),
            49,
            "every key parted on run491 (165 before item 1281's age, 530 before item 1286's city count, \
             967 before item 1275's trade)"
        );
    }

    /// **The word 8786, widened whole** (item 1286):
    /// [`great_sahara_toughest_8786_window`] over run500's blocks
    /// 8781..9037, both directions, every record run491's detail prints.
    /// The word's frame writes block 8787.
    #[test]
    fn run500_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_8786_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 257, "run500 whole: blocks 8781..9037");
        pin!(
            w.missing.is_empty(),
            "run500 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **132 stand on the window's first block** (140 before item 1281's
        // age snap left eight `dest_angle`s): run491's families, and
        // from the gap past its last block, 8433, who=1's group orders —
        // `1/73`..`1/75` carry order id 8698116 here and 8704516 there —
        // and a dozen of its units' `orders_x`/`orders_y`, 24 apart.
        pin_eq!(
            row(1, 73, "order:group.id").as_deref(),
            Some("8781: Group { field: \"id\", ours: 8698116, theirs: 8704516 }"),
            "who=1's group order, standing"
        );
        // The id is the pool's: `(group + frame · 10) · 100 + order_num`
        // with who=1's army group 1 here and pool slot 65 there — both
        // laid on frame 8698 as order 16 (`sim::group::group_move_id`,
        // parked 871).
        //
        // **The word 8786's block 8787** (item 1293): the squad's slot in a
        // wood sent up a flock in the original alone; built, who=1's
        // `flock_stamp` (compared from this item) reads 8786 both sides and
        // the animations of 8789 no longer part.
        pin_eq!(
            row(1, -1, "leader:flock_stamp"),
            None,
            "who=1's flock_stamp agrees on every block"
        );
        // **The word 8856's block 8857** (item 1293) parts on no key first;
        // the nearest first before it is who=1's economy reassembled on
        // 8855 here and last on 8799 there. The two `form` rows are units
        // born in the window (`1/81`, `1/82`), the family that stands on
        // 8781; the human's `production_step` parts on 8801.
        //
        // **Item 1305**: frame 8856's first draw is the Senator `1/80`'s
        // hero coin on its army's turn (`Army::use_generals`), which this
        // crate did not throw. Thrown, 16250 (even, its own cell: no
        // cast), every draw after it lines up again: `1/54`'s
        // `g.cur_anim[0]` ours 30 against 31 and `g.end_time[0]` 70
        // against 80 on 8858 → agreeing, and nothing parts first from
        // 8858 to 8962, where who=1's `peasants` read 34 against 35.
        pin_eq!(
            row(1, -1, "leader:gather_stamp").as_deref(),
            None,
            "who=1's gather_stamp, the first before the word's block"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        // Item 1451, `largest_gather` (AI §115): 8782's three and 8785's eight make-list keys agree.
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= TOUGHEST_WORD_BLOCK_8857 + 3)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            // Item 1377: 8781 81 → 80, who=1's `defense` standing no more.
            [(8781, 50), (8801, 1)],
            "the blocks keys first part on, to three past the word's"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 149.
        // Item 1451, `largest_gather` (AI §115): 91 → 80, the make list from 8782.
        pin_eq!(
            w.firsts.len(),
            51,
            "every key parted on run500 (737 before item 1305, 1,808 before item 1293, \
             1,816 before item 1281)"
        );
    }

    /// **The word 9323, widened whole** (item 1305), and **the word 9352**
    /// on the same blocks (item 1318):
    /// [`great_sahara_toughest_9323_window`] over run511's blocks
    /// 9318..9574, both directions, every record run500's detail prints.
    /// Frame 9323 writes block 9324, and 9352 block 9353.
    #[test]
    fn run511_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_9323_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 257, "run511 whole: blocks 9318..9574");
        pin!(
            w.missing.is_empty(),
            "run511 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The move 9323 → 9352** (item 1318, `docs/AI.md` §99.15): 571
        // keys stood on the window's first block before Forced March's
        // speed was carried — 28 of who=1's army positions among them,
        // `1/69` at (33412, 19136) here and (33056, 18955) there — and on
        // the word's block 9324 the AI scout `1/0` collided with `1/69` in
        // ours alone (`collide_o` 69 against −1). With the march's
        // `UnitData::speed` and the group's `+0x4b`, 133 stand, no
        // position among them; `1/69` agreed to 9432 and `1/0`'s
        // collision to 9362 — and past the move 9352 → 9764, through the
        // window's end.
        pin_eq!(
            row(1, 69, "pos").as_deref(),
            None,
            "the army unit's position, agreeing through the window"
        );
        pin_eq!(
            row(1, 0, "collide_o").as_deref(),
            None,
            "the scout's collision, agreeing through the window"
        );
        // **The move 9352 → 9764** (item 1332, `docs/COLLISION.md` §13.3):
        // the Supply Wagon `1/86` pushes gaia's peacock `8/2` on tick 9347
        // in the original — `detect_boat_collision`'s stranger refusal is a
        // player's only — and ours refused gaia and pushed nothing: `8/2`
        // stood on (38712, 20232) against (38711, 20231) on block 9348,
        // `cur_anim` 0 against 7 on 9349, and on 9355 the wagon collided
        // with it in ours alone (`collide_o` 2 against −1, 40 keys). With
        // the push, the peacock agreed to 9353 and parted `cur_anim` 7
        // against 0 on 9354: the pushed idle unit's guy 0 turn
        // (`turn_angles(bearing, &out, 1, 1)`) left it owing 4° and walking
        // a frame longer. With the turn, `8/2` and `1/86` agree through
        // the window. The Senator `1/80`'s followers parted their step on
        // 9328 (ours 57, theirs 71: `Guy::move`'s crew step takes
        // `GuyData::get_speed`, and this crate took the cached `myspeed`),
        // as they did before either move — until item 1461, which steps
        // them on `Sim::get_speed_at` and leaves the row.
        pin_eq!(
            row(8, 2, "gaia:pos").as_deref(),
            None,
            "gaia's 8/2, pushed on both sides"
        );
        pin_eq!(
            row(1, 86, "collide_o").as_deref(),
            None,
            "the wagon collides with nothing it pushed"
        );
        // Item 1461: its rows leave; a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            row(1, 80, "g.last_speed[2]").as_deref(),
            None,
            "the Senator's crew step"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        // Item 1326 re-pinned on the tree merged with 1318's.
        // Item 1461: its rows leave; a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            by.iter().map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(9318, 47), (9376, 1), (9401, 1)],
            "the blocks keys first part on, the whole window"
        );
        // Item 1326 re-pinned on the tree merged with 1318's: was 1802.
        // Item 1326 re-pinned on the tree merged with 1332's, the make list's `city` compared as the leader's own index: was 159.
        // Item 1354, the `mylos` cache (`docs/VISION.md` §2): 102 → 98, the
        // four rows of block 9533 — the Explorer `1/0` ours 12 against 10
        // and the Caravans `1/22`..`1/24` 7 against 6, ours a frame ahead
        // of the original's refresh.
        // Item 1461: 95 → 78, the Senator `1/80`'s crew rows from 9328 leave; a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            w.firsts.len(),
            49,
            "every key parted on run511 (1,802 before item 1332, 2,069 before item 1318)"
        );
    }

    /// **The word 11882, widened whole** (item 1416):
    /// [`great_sahara_toughest_11882_window`] over run574's blocks
    /// 11876..12133, both directions, every record run571's detail prints.
    /// Frame 11882 writes block 11883.
    #[test]
    fn run574_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_11882_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var_os("RON_FIRSTS").is_some() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 257, "run574 whole: blocks 11876..12133");
        pin!(
            w.missing.is_empty(),
            "run574 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The move 11882 → 11985** (item 1416, `docs/AI.md` §108): who=1's
        // Senate finishes Democracy with The Senator `1/80` already born,
        // and `Build::finished`'s other arm `set_type`s it to The
        // President (353 → 355). On block 11883 `num_units[303]`/`[305]`,
        // `1/80`'s `gpiece` and `des_x/y` parted.
        for (o, key) in [
            (-1, "leader:num_units[303]"),
            (-1, "leader:num_units[305]"),
            (80, "g.gpiece[0]"),
            (80, "g.des_x[1]"),
        ] {
            pin_eq!(row(1, o, key), None, "1/{o}'s {key} agrees");
        }
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= TOUGHEST_WORD_BLOCK_11986 + 1)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            // 128 stand on the window's first block, after the frames
            // between run571's last block and it that no dump prints.
            [(11876, 83), (11976, 1)],
            "the blocks keys first part on, to one past the word's"
        );
        // Item 1426 (`docs/AI.md` §109): the 11985 word agrees after
        // Democracy's research discount. These keys name the price,
        // affordability and purchase, not merely the aggregate improvement.
        for (o, key) in [
            (-1, "leader:MAKE[2].val"),
            (-1, "leader:MAKE[2].t"),
            (-1, "leader:bucket[0:food]"),
            (-1, "leader:bucket[2:wealth]"),
            (2028, "queue:queue[0].cost[0]"),
            (2028, "queue:queue[0].cost[1]"),
            (2046, "build:extra"),
        ] {
            pin_eq!(
                row(1, o, key),
                None,
                "Democracy price/purchase agrees: 1/{o} {key}"
            );
        }
        pin_eq!(
            row(1, 124, "g.angle[0]"),
            Some("11976: ours 1431655765 theirs 0".into()),
            "the Scholar residue remains while the AI word moves"
        );
        // Next measured word: 12538, beyond this capture. The constant
        // stays at its witnessed floor 11985 until item 1429 widens it.
        // Item 1457: 144 → 140; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
        pin_eq!(w.firsts.len(), 91, "every key parted on run574");
    }

    /// **The word 12538, widened whole** (item 1429):
    /// [`great_sahara_toughest_12538_window`] over run584's blocks
    /// 12532..12789, both directions, every record run574's detail prints.
    /// Frame 12538 writes block 12539.
    #[test]
    fn run584_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_12538_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var_os("RON_FIRSTS").is_some() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 257, "run584 whole: blocks 12532..12789");
        pin!(
            w.missing.is_empty(),
            "run584 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The move 12538 → 12569** (item 1429, `docs/AI.md` §118): the
        // Bombard `1/139`'s three draws on 12538 changed order and not
        // value. Its figures' `cur_anim` and `stopped` on block
        // 12539 agree on both sides, before and after the build; so does
        // every other key of the unit.
        for (o, key) in [
            (139, "g.cur_time[0]"),
            (139, "g.cur_anim[0]"),
            (139, "g.cur_anim[1]"),
            (139, "g.cur_anim[3]"),
            (139, "g.stopped[1]"),
            (139, "g.stopped[3]"),
            (139, "g.cur_time[1]"),
            (139, "g.cur_time[3]"),
        ] {
            pin_eq!(row(1, o, key), None, "1/{o}'s {key} agrees");
        }
        // The word's own block: the first state part past the standing
        // keys is who=1's `1/84` group — its formation and id — on 12539.
        pin_eq!(
            row(1, 84, "order:group.form_id"),
            None,
            "1/84's group formation, the first part past the standing keys"
        );
        // **The move 12575 → 12744** (item 1472, `docs/AI.md` §120). The
        // first part past the standing keys of run584's window was `1/109`'s
        // `half_step` on block 12542 (ours 0, theirs 1) and the word 12575
        // stood on the units it set going; run637 (`RON_COLLIDE_PROBE`) printed
        // the original's sweep of 12541 — `collide_here` hit (609, 414) and
        // `is_here(1/101) = 1` — against this crate's hit (611, 414) and no
        // `is_here`, the nine cells `1/84`'s old size-3 block left behind
        // when it became a Bombard on 11833. The one-shot agrees.
        pin_eq!(row(1, 109, "half_step"), None, "1/109's half step agrees");
        pin_eq!(row(1, 101, "half_step"), None, "and so does 1/101's");
        // **The move 12744 → 12816** (item 1477, `docs/AI.md` §122). The
        // word 12744 was a unit born in the original and not here — who=1's
        // `peasants` and `num_units[0]`, and the figures' clocks of the
        // walkers that drew around it — and its first parted field was
        // `1/2031`'s census, standing from block 12532 (and from the city's
        // first census on 9775): `filled` 39 against 38 and `space[0..2]`
        // 46/46/33 against 47/47/34, the circle walk's entry 0 against the
        // original's entry 1 (`plan_strategy@006b9620:0x6bb6f7`). They agree
        // now, and so do who=1's make list on 12582 and the queues on
        // 12583.
        pin_eq!(
            row(1, 2031, "city:filled"),
            None,
            "1/2031's `filled` agrees"
        );
        pin_eq!(row(1, 2031, "city:space[0]"), None, "and its `space[0]`");
        pin_eq!(row(1, 2031, "city:space[2]"), None, "and its `space[2]`");
        pin_eq!(
            row(1, -1, "leader:MAKE[3].val"),
            None,
            "who=1's make list on 12582 agrees"
        );
        pin_eq!(
            row(1, -1, "leader:peasants"),
            None,
            "who=1's census on the old word 12744"
        );
        pin_eq!(
            row(1, 60, "g.cur_anim[0]"),
            None,
            "1/60's figure clock on the old word 12744"
        );
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= TOUGHEST_WORD_BLOCK_12745 + 1)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>()
                .len(),
            // 33 on 1429's own tree, 32 with 1461's crew `get_speed` beside it
            // (the booking gate on 8f83972e); 13 since item 1472; 9 since item 1477.
            9,
            "the blocks keys first part on, to one past the word's"
        );
        // 1694 on 1429's own tree, 1528 with 1461's crew step beside it;
        // 367 since item 1472's crew clear; 142 since item 1477's census walk.
        pin_eq!(w.firsts.len(), 96, "every key parted on run584");
    }

    /// **The word 12816, widened whole** (item 1477):
    /// [`great_sahara_toughest_12816_window`] over run640's blocks
    /// 12811..12834, both directions, every record run584's detail prints.
    /// Frame 12816 writes block 12817.
    #[test]
    fn run640_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_12816_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var_os("RON_FIRSTS").is_some() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 24, "run640 whole: blocks 12811..12834");
        pin!(
            w.missing.is_empty(),
            "run640 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word 12816's block, 12817** (item 1477, `docs/AI.md` §122):
        // the first parts of the window past the 159 standing on its first
        // block are army 65's formation — `curr` and `off` on all 27 slots
        // (slot 0: `off` (−9, −3) here against (−32, −3) there, `curr`
        // (117, 440) against (−127, 1540)) — and `1/141`'s `g.end_time[2]`,
        // 31 against 23, on the frame whose draws are 41 against 40. No
        // mechanism is booked.
        pin_eq!(
            row(1, 141, "g.end_time[2]"),
            None,
            "1/141's third figure's `end_time`, on the word's block"
        );
        pin_eq!(
            row(1, -3, "group:65.curr[0]"),
            None,
            "army 65's formation, slot 0, on the word's block"
        );
        pin_eq!(row(1, -3, "group:65.off[0]"), None, "and its offset");
        pin_eq!(
            by.iter().map(|(b, n)| (*b, *n)).take(3).collect::<Vec<_>>(),
            [(12811, 87), (12825, 1)],
            "the blocks keys first part on, the first three"
        );
        pin_eq!(w.firsts.len(), 88, "every key parted on run640");
    }

    /// **The word 14363, widened whole** (item 1493):
    /// [`great_sahara_toughest_14363_window`] over run653's blocks
    /// 14357..14380, both directions, every record run584's detail prints.
    /// Frame 14363 writes block 14364.
    #[test]
    fn run653_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_14363_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var_os("RON_FIRSTS").is_some() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 24, "run653 whole: blocks 14357..14380");
        pin!(
            w.missing.is_empty(),
            "run653 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word 14363's block, 14364** (item 1493, `docs/AI.md` §128):
        // ours 11 draws and the original 177 on the frame, parting at index
        // 5 (theirs `PathFinder::calc_road_cost+0x46`), and on the block the
        // original's `regen_roads` flag on six of who=1's buildings, `1/66`'s
        // and `1/115`'s order kind 6 against 7. **Item 1503 moved the word
        // 14363 → 14512 and these rows with it** (`docs/AI.md` §132): the
        // Smelter `1/2053` finished on 14363 at the President's clock,
        // `100000 × 100 / 133`, and flagged its city. This test keeps the move's
        // value diff: the window now parts on its first block alone.
        pin_eq!(
            row(1, 2031, "build:regen_roads"),
            None,
            "a city building's road flag, on the word's block"
        );
        pin_eq!(
            row(1, 2053, "build:regen_roads"),
            None,
            "and the one building this crate flagged alone"
        );
        pin_eq!(
            row(1, 66, "order:kind"),
            None,
            "1/66's order kind, on the word's block"
        );
        // **The group record's tail agrees**: the compared slots run to the
        // member count since item 1493, and army 65's `off`/`curr` past its
        // `form_num` carry the siege units' copy-back on both sides.
        pin_eq!(
            row(1, -3, "group:65.off[62]"),
            None,
            "army 65's tail agrees"
        );
        pin_eq!(
            by.iter().map(|(b, n)| (*b, *n)).take(3).collect::<Vec<_>>(),
            [(14357, 88)],
            "the blocks keys first part on, the first three"
        );
        pin_eq!(w.firsts.len(), 88, "every key parted on run653");
    }

    /// **The word 15101, widened whole** (item 1510):
    /// [`great_sahara_toughest_15101_window`] over run663's blocks
    /// 15095..15118, both directions, every record run584's detail prints.
    /// Frame 15101 writes block 15102.
    #[test]
    fn run663_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_15101_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var_os("RON_FIRSTS").is_some() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 24, "run663 whole: blocks 15095..15118");
        pin!(
            w.missing.is_empty(),
            "run663 carries every key: {:?}",
            w.missing
        );
        pin_eq!(
            by.iter().map(|(b, n)| (*b, *n)).take(3).collect::<Vec<_>>(),
            [(15095, 94), (15098, 4)],
            "the blocks keys first part on, the first three"
        );
        pin_eq!(w.firsts.len(), 98, "every key parted on run663");
        // **The word 15101's block, 15102** (item 1510, `docs/AI.md` §136):
        // ours 19 draws against 17, parting at index 7, and block 15102
        // parted on 27 keys of `1/96`, a Bombard (`cur_anim` 24 theirs
        // against 0, 0, 7, 0, `g.angle` -881307648 on every figure,
        // `heading` -881307648 against -632029184, `end_time` 80, 80, 31,
        // 80 against 31, 15, 15, 15). **Moved by item 1517** (`docs/AI.md`
        // §137): `Unit::do_cast`'s unpack sets the unit's angle to guy 0's
        // (`005ecbe0`), for every unit, so the figures unpack where they
        // stand and nobody turns. The 27 keys, and the 165 after them,
        // agree: what stands is the 98 on the window's first two blocks.
        pin_eq!(
            w.firsts
                .iter()
                .filter(|((who, o, _), (f, _))| (*who, *o, *f) == (1, 96, 15_102))
                .count(),
            0,
            "1/96's keys on the word's block"
        );
    }

    /// **Three of Great Sahara's five Bombard rounds strike nothing: their
    /// shooters are decoys** (item 1522, `docs/AI.md` §139). run666 prints
    /// the original's rounds (`AMMO=5`): `1/139`'s lands on 15198, `1/96`'s
    /// on 15209, `1/95`'s on 15213, `1/106`'s on 15217 and `1/84`'s on
    /// 15224, every one on `0/2000` with the same flight. The building's
    /// `damage` moves by 135 on 15198 and 15224 and by nothing on the other
    /// three, and the three that do nothing are the three whose shooter
    /// has `unit_masks & 1` — `Object::do_damage`'s decoy return. The test
    /// reads the dump alone; the code's walk is `run664`'s and `run470`'s.
    #[test]
    fn run666_s_decoy_rounds_strike_nothing() {
        let Some(path) = dump(TOUGHEST_WORD_15213_AMMO) else {
            eprintln!("skipping: no run666 (set RON_GAMELOG_DIR)");
            return;
        };
        let _pins = Pins::hold();
        let mut ix = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
        // Each shooter's last flight block: `cur_time` one short of the
        // `total_time` — the frame that block is numbered by is the landing's.
        let mut landing: std::collections::BTreeMap<i64, i64> = Default::default();
        for f in 15_196..=15_230 {
            let Some(at) = ix.frames().iter().position(|x| x.number == f) else {
                continue;
            };
            let body = ix.read_frame(at).unwrap();
            for (a, _) in crate::diff::ammo::blocks(&body) {
                pin_eq!((a.who, a.whom, a.ox), (1, 0, 2000), "a round on 0/2000");
                pin_eq!(a.total_time, 11, "every flight is eleven frames");
                if a.cur_time == a.total_time - 1 {
                    landing.insert(a.o, f);
                }
            }
        }
        pin_eq!(
            landing.iter().map(|(o, f)| (*o, *f)).collect::<Vec<_>>(),
            [
                (84, 15_224),
                (95, 15_213),
                (96, 15_209),
                (106, 15_217),
                (139, 15_198)
            ],
            "the five rounds and the frame each lands on"
        );
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let at = |n: i64| {
            log.dumps()
                .into_iter()
                .find(|(f, _)| *f == n)
                .or_else(|| log.frames().into_iter().find(|(f, _)| *f == n))
                .map(|(_, b)| crate::gamelog::records(b, false))
                .unwrap_or_default()
        };
        let village = |n: i64| {
            at(n)
                .1
                .iter()
                .find(|b| (b.who, b.o) == (0, 2000))
                .and_then(|b| b.damage)
        };
        // The block numbered by the landing frame is the state before it,
        // the next is the state after.
        let rows: Vec<(i64, bool, i64)> = landing
            .iter()
            .map(|(o, f)| {
                let decoy = at(*f)
                    .0
                    .iter()
                    .find(|u| (u.who, u.o) == (1, *o))
                    .and_then(|u| u.unit_masks)
                    .map(|m| m & 1 != 0)
                    .expect("the shooter is on its landing block");
                (*o, decoy, village(f + 1).unwrap() - village(*f).unwrap())
            })
            .collect();
        pin_eq!(
            rows,
            [
                (84, false, 135),
                (95, true, 0),
                (96, true, 0),
                (106, true, 0),
                (139, false, 135)
            ],
            "shooter, decoy bit, and what the round took off 0/2000"
        );
    }

    /// **The word 15213, widened whole** (item 1517):
    /// [`great_sahara_toughest_15213_window`] over run664's blocks
    /// 15207..15230, both directions, every record run584's detail prints.
    /// Frame 15213 writes block 15214.
    #[test]
    fn run664_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_15213_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var_os("RON_FIRSTS").is_some() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 24, "run664 whole: blocks 15207..15230");
        pin!(
            w.missing.is_empty(),
            "run664 carries every key: {:?}",
            w.missing
        );
        pin_eq!(
            by.iter().map(|(b, n)| (*b, *n)).take(3).collect::<Vec<_>>(),
            [(15207, 92)],
            "the blocks keys first part on, the first three"
        );
        pin_eq!(w.firsts.len(), 92, "every key parted on run664");
        // **The word 15213's block, 15214** (item 1517, `docs/AI.md` §137):
        // ours 8 draws against 7, parting at index 4 — ours
        // `Object::take_damage+0xe1`, theirs `Farms::inc_time+0x1ae`. Item
        // 1522 (`docs/AI.md` §139) found the cause: `1/96`, `1/95` and
        // `1/106` are a General's Bombard decoys (`unit_masks & 1`), and
        // `Object::do_damage` returns at once for a decoy attacker, so their
        // rounds strike nothing; this crate struck `0/2000` for 135 on 15209
        // and 15213 (and the first-wound draw on `0/2005`). With the gate the
        // window parts on **nothing** past the 92 that stand: the 302 keys
        // are 92, the two on 15210 and the one on 15214 are gone, and the
        // word moved to 15275, past every block on disk.
        pin_eq!(
            w.firsts
                .iter()
                .filter(|((who, o, _), (f, _))| (*who, *o, *f) == (0, 2005, 15_214))
                .count(),
            0,
            "0/2005's key on the word's block"
        );
    }

    /// **The word 15275, widened whole** (item 1524):
    /// [`great_sahara_toughest_15275_window`] over run668's blocks
    /// 15230..15433, both directions, every record run584's detail prints.
    /// Frame 15275 writes block 15276.
    #[test]
    fn run668_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_15275_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var_os("RON_FIRSTS").is_some() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 203, "run668 whole: blocks 15230..15432");
        pin!(
            w.missing.is_empty(),
            "run668 carries every key: {:?}",
            w.missing
        );
        pin_eq!(
            by.iter().map(|(b, n)| (*b, *n)).take(5).collect::<Vec<_>>(),
            [(15230, 92), (15266, 1), (15268, 1), (15326, 1), (15366, 1)],
            "the blocks keys first part on, the first five"
        );
        pin_eq!(w.firsts.len(), 2596, "every key parted on run668");
        // **The word 15275's frame, widened** (item 1524, `docs/AI.md`
        // §140): ours 12 draws against 11, parting at index 4 — ours
        // `Guy::set_anim+0x97a < Unit::move_step+0x823`, theirs
        // `… < Unit::do_guard+0x7f4`. The first parted field is `1/153`'s
        // move order on block 15247 (frame 15246): the original takes a
        // new waypoint, (9480, 27240), refuses it to the General's Bombard
        // decoy `1/106` and writes `coll_x/coll_y`; ours' scan let the two
        // slip past on the corner rule, because `UnitData::is_corner` walks
        // `0 .. guy_mark` (1) and ours walked the decoy's three crew
        // figures too. With the walk bounded the unit has **no key at
        // all** on the window and the word moved to 15378.
        pin_eq!(
            w.firsts
                .keys()
                .filter(|(who, o, _)| (*who, *o) == (1, 153))
                .count(),
            0,
            "1/153 parts on nothing"
        );
        pin_eq!(
            by.get(&15_276).copied().unwrap_or(0),
            0,
            "nothing parts on the old word's block"
        );
        pin_eq!(
            by.get(&15_378).copied().unwrap_or(0),
            887,
            "the new word's block (frame 15377)"
        );
    }

    /// **The word 14512, widened whole** (item 1503):
    /// [`great_sahara_toughest_14512_window`] over run659's blocks
    /// 14506..14529, both directions, every record run584's detail prints.
    /// Frame 14512 writes block 14513.
    #[test]
    fn run659_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_14512_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var_os("RON_FIRSTS").is_some() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 24, "run659 whole: blocks 14506..14529");
        pin!(
            w.missing.is_empty(),
            "run659 carries every key: {:?}",
            w.missing
        );
        // **The word 14512's block, 14513** (item 1503, `docs/AI.md` §132):
        // 53 draws on each side, parting at index 40 — ours `Guy::move+0x19f`,
        // theirs `Guy::turn_towards+0x69` — in `1/141`'s four figures (a
        // BOMBARD, move/move/turn/move there, move/move/move/turn here). **No
        // dumped record parts on the frame or after it**: every one of the 83
        // keys stands from the window's first block, so the delta is a draw
        // order whose values the dump does not print. No mechanism is booked.
        // **Moved by item 1510** (`docs/AI.md` §136): a tracked crew figure
        // runs `Guy::move` in its slot, so guy 2's turn is the third draw;
        // the 83 keys and this block's agreement are unchanged, the clocks
        // being re-seated from the dump on every traced frame.
        pin_eq!(
            by.iter().map(|(b, n)| (*b, *n)).take(3).collect::<Vec<_>>(),
            [(14506, 83)],
            "the blocks keys first part on, the first three"
        );
        pin_eq!(w.firsts.len(), 83, "every key parted on run659");
    }

    /// **The word 11182, widened whole** (item 1379):
    /// [`great_sahara_toughest_11182_window`] over run571's blocks
    /// 11177..11433, both directions, every record run562's detail prints.
    /// Frame 11182 writes block 11183.
    #[test]
    fn run571_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_11182_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var_os("RON_FIRSTS").is_some() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 257, "run571 whole: blocks 11177..11433");
        pin!(
            w.missing.is_empty(),
            "run571 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The move 11182 → 11382** (item 1388, `docs/AI.md` §102): the
        // first library's queue advances one slot per city holding a
        // library, so who=1's Trade and Conscription run together as the
        // original's do. who=1's make list on 11181 (the Scholars and the
        // Citizens at 9,999,999) and the word 11182's block 11183 agree.
        for key in [
            "leader:production_step",
            "leader:epoch[0]",
            "leader:epochs",
            "leader:pop_cap",
        ] {
            pin_eq!(row(1, -1, key), None, "who=1's {key} agrees");
        }
        pin_eq!(
            row(1, 2005, "queue:queue[0].job_counter"),
            None,
            "the first library's research queue runs in parallel"
        );
        // **The word 11382's block 11383** (item 1388): ours spends two
        // `Leader::use_market+0x1ed` where the original spends
        // `make_stuff`'s own, and who=1's make list parts on 11381 — ours
        // `t` 61 (a Merchant) at 931034, the original's the Bombard at
        // 604160; `caras` ours 3 against 4 on 11372.
        pin_eq!(
            row(1, -1, "leader:MAKE[2].t").as_deref(),
            None,
            "who=1's make list, its third"
        );
        pin_eq!(
            row(1, -1, "leader:MAKE[2].val").as_deref(),
            None,
            "who=1's make list, its third's value"
        );
        pin_eq!(
            row(1, -1, "leader:caras").as_deref(),
            None,
            "who=1's caravans"
        );
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= TOUGHEST_WORD_BLOCK_11383 + 1)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            // 120 stand on the window's first block, after 146 frames no
            // dump of this game prints (11031..11176).
            [(11177, 80), (11201, 1), (11376, 1)],
            "the blocks keys first part on, to one past the word's"
        );
        // Measured on the tree after 1388's change.
        pin_eq!(w.firsts.len(), 82, "every key parted on run571");
    }

    /// **The word 10779, widened whole** (item 1371):
    /// [`great_sahara_toughest_10779_window`] over run562's blocks
    /// 10774..11030, both directions, every record run547's detail prints.
    /// Frame 10779 writes block 10780.
    #[test]
    fn run562_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_10779_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var_os("RON_FIRSTS").is_some() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 257, "run562 whole: blocks 10774..11030");
        pin!(
            w.missing.is_empty(),
            "run562 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word 10779's block 10780** (item 1371): ours spent a fifth
        // `Leader::upgrade_units+0x5a4` roll where the original spends
        // four — the Dragoon, whose Heavy Horse Archers rung was in
        // research. **Item 1379** gave `upgrade_units` `researching`'s
        // unit arm, and the block agrees whole: who=1's `MAKE[0].val` ours
        // 1152000 against 672000, `MAKE[2].cat` 7 against 8 and `0/3`'s
        // `orders_x` 5112 against 5304 → agreeing. The make list's values
        // part again from 10782 (`MAKE[3].val` 270400 against 276800) with
        // no draw spent on them; the head's first parting is 10785.
        // Item 1451, `largest_gather` (AI §115): the head still parts on 10785, ours 51046 → 43828 against 44890.
        pin_eq!(
            row(1, -1, "leader:MAKE[0].val").as_deref(),
            None,
            "who=1's make list, its head"
        );
        pin_eq!(
            row(1, -1, "leader:MAKE[3].val").as_deref(),
            None,
            "who=1's make list, its fourth entry: the first row past 10774"
        );
        pin_eq!(
            row(1, -1, "leader:MAKE[2].cat"),
            None,
            "who=1's make list, its third entry's category"
        );
        pin_eq!(row(0, 3, "orders_x"), None, "0/3's walk, the shifted roll");
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= TOUGHEST_WORD_BLOCK_10780 + 1)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            // 117 stand on the window's first block, after 378 frames no
            // dump of this game prints (10396..10773) — 119 before item
            // 1388, whose parallel research took `1/2005`'s two queue rows;
            // nothing parts on the word's block or the next.
            [(10774, 76)],
            "the blocks keys first part on, to one past the word's"
        );
        // Measured on the tree merged with 1377's: 1,803 → 1,802 by item
        // 1377 (who=1's `defense` on 10929), → 144 by item 1379, → 130 by
        // item 1388 (the research queue's two rows, `epoch[2]`, `epochs`,
        // `queued`, `resource_cap` ×5 and `MAKE[3].t`).
        // Item 1451, `largest_gather` (AI §115): 126 → 125, MAKE[3].t.
        pin_eq!(w.firsts.len(), 83, "every key parted on run562");
    }

    /// **The word 10144, widened whole** (item 1354):
    /// [`great_sahara_toughest_10144_window`] over run547's blocks
    /// 10139..10395, both directions, every record run529's detail prints.
    /// Frame 10144 writes block 10145.
    #[test]
    fn run547_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_10144_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var_os("RON_FIRSTS").is_some() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 257, "run547 whole: blocks 10139..10395");
        pin!(
            w.missing.is_empty(),
            "run547 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The move's value diff on block 10145** (item 1365, `docs/PRODUCTION.md`,
        // "The tail's first caller"): the original trains the Citizen `1/89`
        // out of `1/2022` on frame 10144 at a target of 16,200 — the troops'
        // speed upgrade, Herbal Lore held — and ours now does too. Before it
        // the Citizen was the original's alone and `1/2022`'s `queue:queued`
        // ours 1 against 0 (item 1354's word block).
        pin_eq!(
            row(1, 89, "unlinked"),
            None,
            "the Citizen 1/89, born on both sides"
        );
        pin_eq!(
            row(1, 2022, "queue:queued"),
            None,
            "1/2022's queue, emptied on both sides"
        );
        // The census's newborn lag (parked 1122): `Unit::set_type` adds the
        // Citizen to `peasants` at once, and this crate counts it at the
        // next sweep — `peasant_high` is 38 on both sides.
        pin_eq!(
            row(1, -1, "leader:peasants").as_deref(),
            None,
            "who=1's citizens, the newborn lag"
        );
        // **The move's value diff on the word 10391** (item 1371,
        // `docs/COLLISION.md` §5.1): the caravan `1/52` takes its last
        // node on frame 10390 with the Citizen `1/87` standing on it, and
        // under a `TRADE_ROUTE` the waypoint take kills the move where it
        // stands. Before it ours widened the tolerance to 144 and walked
        // on: on block 10391 its `pos` ours (28260,24052) against
        // (28261,24026), `orders.len` 2 against 1, `collide_o` -1 against
        // 87 (item 1365's word block, 10392).
        for what in ["pos", "orders.len", "collide_o", "tolerance", "orders_x"] {
            pin_eq!(row(1, 52, what), None, "1/52's {what}, agreeing");
        }
        pin_eq!(
            by.iter().map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            // Between the newborn lag and the window's end, one row each:
            // who=1's `gather_stamp` 10151 against 10023 (10152),
            // who=0's `production_step` 0 against 1 (10201), who=1's
            // `MAKE[3].val` 276800 against 283200 (10382) — and nothing
            // from 10383 to 10395. Its `defense` 1 against 2 on 10178
            // left with item 1377 (`Build::init`'s `+1`).
            [(10139, 47), (10201, 1)],
            "the blocks keys first part on, the whole window"
        );
        // Item 1371: 156 → 87, the caravan's 39 + 19 + 9 + 2 gone
        // (measured on the tree merged with 1358's). Item 1377: 87 → 86,
        // who=1's `defense` on 10178.
        pin_eq!(
            w.firsts.len(),
            48,
            "every key parted on run547 (156 before item 1371)"
        );
    }

    /// **The word 9764, widened whole** (item 1332):
    /// [`great_sahara_toughest_9764_window`] over run529's blocks
    /// 9759..10015, both directions, every record run500's detail prints.
    /// Frame 9764 writes block 9765.
    #[test]
    fn run529_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_9764_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 257, "run529 whole: blocks 9759..10015");
        pin!(
            w.missing.is_empty(),
            "run529 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The move 9764 → 9982** (item 1338, `docs/ANIM.md` §11, §4):
        // the Catapult `1/84` became a Trebuchet (`TypeIndex` 266) on 9710,
        // and its crew is fresh and trackless in the original — ours kept
        // the Catapult's tracks (−120, 0) and (72, 216) and walked them
        // (`g.cur_anim[1]` 8 against 22 on 9759); and its guy 0, still
        // turning on slot 22, rolled an idle every frame where the
        // original's rolls on 9755 and 9764 only (`g.cur_time[0]` 1
        // against 4 on 9759). The word's own row, `g.cur_time[2]` 10
        // against 1 on 9765, agrees with both.
        for key in ["g.track_dx[1]", "g.cur_time[0]", "g.cur_time[2]"] {
            pin_eq!(row(1, 84, key), None, "the Trebuchet's {key} agrees");
        }
        // Its guy 1's slot parted again on 10004, past the word 9999 (item
        // 1346), and agrees since the move 9999 → 10144 (item 1354).
        pin_eq!(
            row(1, 84, "g.cur_anim[1]"),
            None,
            "the Trebuchet's g.cur_anim[1] agrees"
        );
        // **The move 9982 → 9999** (item 1346, `docs/AI.md` §99.16): the
        // Gunpowder Age at 382 with Silver, bought by the cheap tick of
        // frame 9955 off the head's escrowed purse. Block 9956's leader rows
        // and the word 9982's block 9983 rows agree.
        for key in [
            "leader:MAKE[0].val",
            "leader:bucket[0:food]",
            "leader:bucket[3:knowledge]",
            "leader:escrow[0:food]",
            "leader:escrow[3:knowledge]",
            "leader:escrow[1:timber]",
            "leader:MAKE[5].t",
        ] {
            pin_eq!(row(1, -1, key), None, "who=1's {key} agrees");
        }
        pin_eq!(
            row(1, 2005, "queue:queued"),
            None,
            "the age queued at `1/2005`"
        );
        // **The move 9999 → 10144** (item 1354, `docs/VISION.md` §2): who=1
        // takes Herbal Lore (`TROOPS_LOS_1`) on 9782, and every Barracks
        // and Stable unit it owns sees two tiles more from block 9784 —
        // the next leader pass after `gain_tech`'s `|= 0xc000000`. The
        // value diff: on 9784 the Explorer `1/0`'s `mylos` ours 12 against
        // 14, `1/28`'s 11 against 13 and `1/38`'s 8 against 10, 29 keys,
        // → agreeing; the Explorer's path on 9839 (`path[1].to` (26616,
        // 13560) against (26616, 14328)) and its `pos` from 9840 →
        // agreeing; the word 9999's block 9999, `orders.len` ours 0
        // against 1 → agreeing, and the `tolerance` from 9943 with it.
        for (o, key) in [
            (0, "mylos"),
            (28, "mylos"),
            (38, "mylos"),
            (0, "path[1].to"),
            (0, "pos"),
            (0, "orders.len"),
            (0, "tolerance"),
        ] {
            pin_eq!(row(1, o, key), None, "1/{o}'s {key} agrees");
        }
        // The new word, 10144, is past this window's last block 10015.
        // Item 1461: its rows leave; a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            by.iter().map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [
                (9759, 49),
                (9765, 1),
                (9781, 1),
                (9782, 1),
                (9799, 1),
                (9801, 1),
                (9824, 1),
                (9993, 1),
                (9994, 1),
                (10011, 1),
                (10014, 1)
            ],
            "the blocks keys first part on, the whole window"
        );
        // Item 1338, on the tree merged with 1326's: 1,075 → 557.
        // Item 1330, on the tree merged with 1338's: 557 → 505, the births' `form` (`docs/GROUPS.md` §24.3).
        // Item 1346, on the tree merged with 1330's: 505 → 262.
        // Item 1354, the troops term and the `mylos` cache: 262 → 114.
        // Item 1377, who=1's `defense` standing on 9759 no more: 114 → 113.
        // Item 1461: 110 → 95; a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(w.firsts.len(), 59, "every key parted on run529");
    }

    /// **The gap 9038..9317, widened whole** (item 1318):
    /// [`great_sahara_toughest_gap_window`] over run517's blocks
    /// 9032..9323, both directions, every record run500's detail prints —
    /// the frames the 571 keys that stood on run511's first block parted
    /// in, and the move 9323 → 9352's value diff.
    #[test]
    fn run517_s_gap_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_gap_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 292, "run517 whole: blocks 9032..9323");
        pin!(
            w.missing.is_empty(),
            "run517 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The move 9323 → 9352's value diff** (item 1318, `docs/AI.md`
        // §99.15). The Senator `1/80` cast Forced March on 9112, and with
        // the march's speed not carried the gap parted on **block 9114**:
        // 179 keys, every one of the army's 26 walkers — `1/28`'s `pos`
        // ours (32179, 18050) against (32189, 18050), its guy's
        // `last_speed` 15 against 25, `1/29`'s 31 against 52 — and on 9116
        // the Senator's group 65's `speed` and `new_speed` 41 against 42,
        // the march's `FORCED_MARCH_SPEED`. With it carried, none of them
        // parts: 158 keys where 1,154 parted.
        pin_eq!(row(1, 28, "pos"), None, "an army walker, agreeing");
        pin_eq!(
            row(1, 29, "g.last_speed[0]"),
            None,
            "a walker's step, agreeing"
        );
        pin_eq!(
            row(1, -3, "group:65.speed"),
            None,
            "the Senator's group walks at 42"
        );
        // What still parts in the gap: group 67's speed from 9084 (ours 0
        // against 34), before the march; and the Senator's crew step from
        // 9222 (ours 57 against 71, `docs/MOVEMENT.md`, "the crew guy's
        // step speed is the cached value").
        // Item 1457: Some("9084: ours 0 theirs 34") → None; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
        pin_eq!(
            row(1, -3, "group:67.speed").as_deref(),
            None,
            "group 67's speed, before the march"
        );
        // Item 1461: its rows leave; a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            row(1, 80, "g.last_speed[1]").as_deref(),
            None,
            "the Senator's crew step"
        );
        // Item 1326 re-pinned on the tree merged with 1318's.
        // Item 1457: [(9032, 76), (9084, 2), (9201, 1), (9222, 4), (… → [(9032, 76), (9201, 1), (9222, 4), (9223, 1), (…; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
        // Item 1461: its rows leave; a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            by.iter().map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(9032, 47), (9201, 1)],
            "the blocks keys first part on"
        );
        // Item 1326 re-pinned on the tree merged with 1318's: was 158.
        // Item 1338: 151 → 150. The Scout `1/0` is upgraded to an Explorer
        // (`TypeIndex` 71) on 9306, and its fresh crew figure is seated at
        // guy 0's angle: `1/0`'s `g.angle[1]` on 9307, ours −402259968
        // against −363239852 → agreeing (`docs/ANIM.md` §11).
        // Item 1457: 95 → 93; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
        // Item 1461: 93 → 77, the crew rows from 9222 leave; a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            w.firsts.len(),
            48,
            "every key parted on run517 (1,154 without the march's speed)"
        );
    }

    /// **The goods, every frame to the first word's block** (item 1251):
    /// [`great_sahara_toughest_goods_window`] over run482's blocks 1..5378,
    /// both directions, every field `LEADERS=2` prints for every leader.
    #[test]
    fn run482_s_goods_are_widened_whole() {
        let _pins = Pins::hold();
        let Some(w) = great_sahara_toughest_goods_window() else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
        }
        pin_eq!(w.blocks, 5_377, "run482 whole: blocks 1..5378");
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The value diff, block 4577** (frame 4576, the script step that
        // researches Empire): who=1's food stood ours 161 against 197 from
        // here to the word 5782 — Empire at 144 food here, 108 there, the
        // quarter Dye takes off a civic epoch (`docs/AI.md` §99.8). With
        // the arm the whole goods record agrees on every block.
        pin_eq!(
            row(1, -1, "leader:bucket[0:food]"),
            None,
            "who=1's food agrees through 4577"
        );
        // **What parts is not the goods**: the human's `filled_gather_slots`
        // from block 1 (a standing row every widening of this game carries),
        // and the building and group records `LEADERS=2` does not print,
        // which this crate holds alone.
        let leader: Vec<_> = w
            .firsts
            .keys()
            .filter(|(_, o, what)| *o == -1 && what.starts_with("leader:"))
            .map(|(who, _, what)| format!("{who} {what}"))
            .collect();
        pin_eq!(
            leader,
            [
                "0 leader:filled_gather_slots[0:food]",
                "0 leader:filled_gather_slots[1:timber]"
            ],
            "the leader rows that part on run482"
        );
        pin_eq!(w.firsts.len(), 40, "every key parted on run482");
    }

    /// **The goods through the gap, 6030..7066** (item 1275):
    /// [`great_sahara_toughest_wealth_window`] over run495's blocks, both
    /// directions, every field `LEADERS=2` prints for every leader.
    #[test]
    fn run495_s_goods_are_widened_whole() {
        let _pins = Pins::hold();
        let Some(w) = great_sahara_toughest_wealth_window() else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
        }
        pin_eq!(w.blocks, 1_037, "run495 whole: blocks 6030..7066");
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The value diff, block 6592** (frame 6591): who=1's wealth
        // income stood ours 992 against 1000 from here, its `leftover` with
        // it and its bucket from 6614 — caravan `1/52` took city 2 ↔ 3 on
        // 6587, and the original's `do_trade` re-summed city 2's delivered
        // route to city 1 at today's value, 176 → 184 (`docs/AI.md`
        // §99.10). With the recompute the goods agree on every block.
        pin_eq!(
            row(1, -1, "leader:income[2:wealth]"),
            None,
            "who=1's wealth income agrees through 6592"
        );
        let leader: Vec<_> = w
            .firsts
            .keys()
            .filter(|(_, o, what)| *o == -1 && what.starts_with("leader:"))
            .map(|(who, _, what)| format!("{who} {what}"))
            .collect();
        pin_eq!(
            leader,
            [
                "0 leader:filled_gather_slots[0:food]",
                "0 leader:filled_gather_slots[1:timber]"
            ],
            "the leader rows that part on run495"
        );
        pin_eq!(
            w.firsts.len(),
            42,
            "every key parted on run495 (46 before the trade)"
        );
    }

    /// **Every road search to 7070, node for node** (item 1260). run483's
    /// trace proxies `valid_roadcoord` and `calc_road_cost` over the whole
    /// game, so each search's priced nodes are comparable with ours from
    /// frame 0: 49 frames of them to the old word, the last caravan `1/52`'s
    /// replan of its road from `1/2007` to `1/2022`. It parted at node 1129
    /// on 7070 — tile (150, 123) 114 against 37, a road the original still
    /// had and ours had swept away on 6949 — until `leech_codes` joined the
    /// tile beside the Farm's footprint back to its run (`docs/ROADS.md`
    /// §11, `docs/AI.md` §99.9).
    #[test]
    fn run483_s_road_searches_hold_node_for_node_to_7070() {
        let _pins = Pins::hold();
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(t483)) = (
            dump(TOUGHEST_LONG.0),
            dump(TOUGHEST_START),
            trace(TOUGHEST_LONG.1),
            trace("rontrace-run483.log"),
        ) else {
            eprintln!("skipping: no run470/run468/run483 (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        built.sim.trace_costs = true;
        let mut searched = Vec::new();
        let mut parted = Vec::new();
        while built.sim.frame <= TOUGHEST_ROAD_FRAME_7070 {
            let f = built.sim.frame;
            built.sim.road_marks.clear();
            built.tick();
            let ours = std::mem::take(&mut built.sim.road_marks);
            let theirs = t483.road_nodes(f);
            if ours.is_empty() && theirs.is_empty() {
                continue;
            }
            searched.push((f, theirs.len()));
            if let Some(i) =
                (0..ours.len().max(theirs.len())).find(|&i| ours.get(i) != theirs.get(i))
            {
                parted.push(format!(
                    "{f} at {i}: ours {:?} theirs {:?} ({} against {})",
                    ours.get(i),
                    theirs.get(i),
                    ours.len(),
                    theirs.len()
                ));
            }
        }
        pin!(parted.is_empty(), "a road search parted: {parted:?}");
        pin_eq!(searched.len(), 49, "the road-search frames to 7070");
        pin_eq!(
            searched.last().copied(),
            Some((TOUGHEST_ROAD_FRAME_7070, 2_543)),
            "7070's search arrives on its frame"
        );
        // The endpoints, from `astar_caravan_road`'s own bracket: caravan
        // slot 2's route, the second city to the third.
        let bracket = t483.calls_in(TOUGHEST_ROAD_FRAME_7070, 5);
        pin_eq!(
            bracket
                .iter()
                .map(|c| (c.args[1], c.args[2], c.args[3], c.args[5]))
                .collect::<Vec<_>>(),
            [(2007, 1, 2022, 2)],
            "one road plan on 7070, 1/2007 to 1/2022 for caravan slot 2"
        );
    }

    /// **The third map's word at Toughest** (item 1221): run470, the draw
    /// stream at `cover=0` to the game's end, whose first 1,851 frames are
    /// run469's word for word (its stanza's `rngcmp.py` check), walked from
    /// run468's start.
    #[test]
    fn run470_is_great_sahara_at_toughest_and_its_word_holds() {
        let Some(w) = walk_sahara_from(TOUGHEST_START, TOUGHEST_LONG) else {
            return;
        };
        assert_eq!(w.difficulty, 5, "run470's GAME INFO reads DIFFICULTY 5");
        assert_eq!(
            w.last,
            ai_word_length("GreatSaharaToughest"),
            "run470's trace runs to the length `AI_WORDS` gives the game"
        );
        assert!(
            w.count >= THIRD_WORD_GREAT_SAHARA_TOUGHEST
                && w.sequence >= THIRD_WORD_GREAT_SAHARA_TOUGHEST,
            "the third map's word at Toughest fell: count {}, sequence {} of {} — the \
             floor is {THIRD_WORD_GREAT_SAHARA_TOUGHEST}; {}",
            w.count,
            w.sequence,
            w.last,
            w.row
        );
    }
}
