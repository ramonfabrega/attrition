# Jev evidence experiments: exhaustive search, uncertainty, and a prototype

## Outcome

The useful extension is **cheap exhaustive semantic search with several questions
per passage**. On a small authored query set, it recovered nominated evidence
outside a lexical shortlist. A reusable opt-in search tool now emits source-linked
passages and caches exact requests. It does not answer questions or modify code.

The harder evidence tests also narrowed the earlier enthusiasm: classifying a
claim and finding the citations needed to justify it are different tasks. Jev
can classify uncertainty while omitting the very source that explains the gap.
Splitting citation questions into support, contradiction and limitation improves
recall, but returns more extra sources. No autonomous acceptance check is proposed.

No simulator, main-loop document, capture setting, or fidelity score changed.
This work follows commit `30b5bae` on `codex/jev-lab`.

## Cost, protocol, and retained evidence

The user authorized exploration up to **$1 total**, including permission to send
project source and authored reports. Phase two made **401 HTTP requests**, received
**4,536 typed answers**, and used **846,906 input tokens**: estimated **$0.035570**.
Cumulative estimated spend, including phase one and its connectivity probe, is
**$0.048888**. The remaining budget is a ceiling, not a spending target.

All requests pin `jev-1.13.0`. Cost uses the documented $0.042/million input tokens;
output tokens are free. This is reported usage multiplied by published price,
not an invoice. The shared ledger reserves $0.003 before each dispatch, exceeding
65,536 input tokens at that price. Concurrent reservations use a thread lock and
file lock; failed/uncertain calls retain their reserve. Successful validated
usage replaces the reservation. Preserve `/tmp/jev-evidence-budget.json` when
continuing: this is a ledger for these scripts, not an account-wide spending cap.

The [compressed archive](2026-09-19-jev-evidence-observations.json.gz) retains all
seven runs, exact requests/responses, labels, per-call elapsed times,
source text, model IDs, usage, and the budget snapshot. Repeated state is interned
by SHA-256. Every request manifest was written before its run; the offline
analyzer reconstructs and verifies its exact hash. This prevents accidental
post-run question/label changes from hiding in a report. It does not make the
author's labels independently correct. No credential or installed-game file is
included. Only our authored reports and tracked project source were submitted.

## Larger-corpus retrieval

Corpus: **159 overlapping passages from 50 pre-existing lab reports**, excluding
the Jev reports, history and ledger. Character windows are 3,000 with a 2,200
stride. There are 20 fresh investigator-authored queries: 16 with nominated
source files, four asking for achievements these reports have not established.
Four of the answerable questions nominate two complementary sources.

The first method ranks all passages with BM25 (k1=1.5, b=0.75), then gives its
top 12 passages to Jev for relevance judgments. Two queries omit a nominated
file entirely: its earliest passage is rank **55** for acquisition loss checks,
and rank **41** for explicit-install orchestration. No reranker can recover a
passage it never sees. Other passages may supply overlapping evidence, so this
is a nominated-source miss, not proof that every possible answer was lost.

The exhaustive method asks all 20 relevance questions about each passage in
one request, with four requests in flight. It covers the entire corpus instead
of relying on BM25 recall. Scores are ranked by passage and deduplicated by
file; file score is its highest passage score.

| All nominated files recovered within | BM25 | Exhaustive Jev |
| --- | ---: | ---: |
| First file | 6/16 | 11/16 |
| First three files | 12/16 | 14/16 |
| First five files | 12/16 | 15/16 |

Single-file recovery cannot satisfy the four two-source queries. The exhaustive
scan used **376,109 input tokens**, estimated **$0.015797**, and **24.17 seconds**.
It promoted the acquisition-loss report from BM25 passage rank 55 to file rank 1.
The remaining five-file miss concerns consecutive traces versus acquisition
endpoint completeness. Relevant alternative sources also exist; nominated-file
recall deliberately does not claim exhaustive relevance labeling.

These are curated questions from the investigator who knows the documents,
using an untuned lexical baseline. No independent query set, embedding baseline,
reasoning-model comparison, or repeated latency benchmark exists. Exhaustive
scoring is affordable at this corpus size; that does not establish its scaling
to an entire large repository. High relevance is not support for a query's
premise: all four unestablished-achievement queries still have relevant passages
explaining limitations. Do not turn a relevance threshold into an answerability
decision.

The shortlist's separate answerability prompt scored 19/20 under the initial
authored labels. Its one disagreement asks for evidence of a negative graphics
claim, while the instruction privileges demonstrated positive results. That
prompt/label tension is unresolved; **19/20 is not an adoption metric**.

## Evidence packets and citation recall

Sixteen fictional packets cover current versus old revisions, retracted notes,
equal-authority conflicts, scope limits, missing observations, counterexamples,
vacuous comparisons and split evidence. Each is run as written, reordered, with
nearby distractors, and with long irrelevant padding. These are 16 underlying
cases, not 64 independent cases.

All four variants scored **15/16** on the initial four-way verdict. The repeated
disagreement concerns an unrun map in a claim that both comparisons agree.
That wording can imply completed comparisons; it is not a clean demonstration
of inability to reason about missing evidence.

A 24-question follow-up separated behavioral equivalence from whether tests
were executed. Its first version scored 18/24 but omitted explicit revision
binding and used ambiguous "validation" wording. Those six apparent failures
are not reliable model-error labels. The corrected packet states that every
observation is at R3 and explicitly distinguishes execution from agreement:
**24/24**. Standard and explanatory instructions both pass. Both versions are
retained, so this correction does not rewrite the experiment's history.

The citation task has a clearer operational distinction. Initial questions ask
whether each source belongs to the minimal decisive evidence, explicitly
including missing coverage. Separate facet questions instead ask whether a
source supports, contradicts, or states a relevant limitation. The proposed
collection rule is the maximum of those three scores at a fixed 0.5 threshold.

| Variant | Initial recovered / 18 nominated citations | Facets recovered / 18 | Initial extra / facet extra |
| --- | ---: | ---: | ---: |
| Base | 13 | 18 | 1 / 2 |
| Reordered | 12 | 18 | 1 / 2 |
| Nearby distractors | 13 | 18 | 1 / 12 |
| Long padding | 17 | 18 | 2 / 20 |

The initial misses include an unperformed causal test, finite input coverage,
an unavailable memory measurement, and absence of current-revision execution.
These are precisely passages a reader needs to justify uncertainty.

Extra citations are relative to authored minimal sets, not all false positives:
for example, a source supporting one half of a refuted conjunction may remain
useful context. Generic planning notes also receive scores despite supplying no
measurement. The facet strategy trades selectivity for recall; it is not a
strictly superior replacement or a calibrated confidence composition. The
model cannot see answers to sibling questions in the same request.

## Opt-in source-search prototype

`tools/explore/jev_search.py` accepts a JSON map of research questions and
explicit Git path patterns. It reads matching tracked text, rejects symlinks,
keeps line-addressed overlapping windows, and asks all questions about each
window. It emits ranked snippets and Markdown source links. Model judgments are
presented as relevance scores, not proofs. Exact model/request hashes key a
local cache; changed text, questions or positions invalidate it. Incomplete
runs report their error counts and must not be treated as exhaustive results.

A smoke run used **46 windows from eight selected Rust/Python files** and six
implementation questions. Direct inspection confirmed the leading windows
contain the relevant implementation: unobserved fixture rejection, finalized
frame validation, activity/replay digest separation, no-float checking,
metadata-based observation caching and scoreboard/floor comparison. This is
a selected-file navigation demonstration, not repository-wide benchmark accuracy.

Wall time was **7.14 seconds** live; an identical repeat had **46/46 cache hits**,
no API calls or credential retrieval, and a 0.0034-second measured inner search.
That excludes interpreter startup and preparation; the cache does not make novel
queries free. The retained archive includes both the original source-search
responses and the cached result. Line spans round-trip against the read files.

Example with the retained questions (change the glob selection to control scope):

```sh
python3 tools/explore/jev_search.py \
  --queries tools/explore/jev_source_queries.json \
  --glob 'crates/sim/src/no_float.rs' \
  --glob 'crates/sim/src/soak.rs' \
  --glob 'tools/release_gate.py' \
  --output /tmp/new-jev-source-search
```

That short command scans only three files and cannot answer all six questions;
the complete eight-file smoke manifest is retained in the archive. Add
`--dry-run` to inspect the exact prepared requests without retrieving credentials
or sending data. Credentials come from the authorized Passage entry or an
environment variable, never a committed file.

## Verification and next decision

```sh
python3 tools/explore/jev_evidence_analyze.py
python3 -m unittest discover -s tools/explore -p test_jev_evidence_lab.py -v
```

Five offline tests cover concurrent budget rejection, uncertain-call reservation,
invalid token accounting, complete source-window coordinates, and cache-only
execution plus changed-source rejection. The budget test was deliberately run
with rejection disabled in a temporary module: it failed because all eight
concurrent calls were admitted instead of one. The repository source was never
mutated. Frozen manifest and state-hash verification passes for every live run.

The next useful trial is a real, unfamiliar research question with a small
predeclared corpus and an independent reader judging whether the retrieved
passages include decisive qualifications. The prototype is ready for that trial;
there is no reason to spend the rest of the dollar merely accumulating easy
synthetic cases.

The full explicit-install release gate passed: 332 rondata tests (two ignored),
855 sim tests, 13 fixed tests and three doctests, plus offline Python tests,
install survey, clippy, formatting and paperwork guards. The fixture audit
recorded 782 requests with zero missing. Sampled peak was 12,162 MiB under the
20 GiB process-tree cap. Command:
`python3 tools/release_gate.py /Users/rf-studio/code/fun/attrition/game --test-threads 4 --report-dir /tmp/jev-evidence-release-gate`.
Output: `/tmp/jev-evidence-release-gate.log`. The five new opt-in Python tests
were run separately; they are deliberately not registered in the main gate.

API design references: [TypeSafe reranking cookbook](https://docs.typesafe.ai/cookbooks/rerank_typesafe),
[citation checking](https://docs.typesafe.ai/cookbooks/citation_check),
[Noul](https://docs.typesafe.ai/primitives/noul),
[current model and price](https://docs.typesafe.ai/models), checked 2026-09-19.
