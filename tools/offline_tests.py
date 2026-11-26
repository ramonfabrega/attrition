#!/usr/bin/env python3
"""Run the reviewed offline acquisition and gate tests without an install.

Use explicit module names: new experimental tests are not discovered implicitly.
The cleanup suite mocks live-session operations and uses authored temporary
profiles; its launch-argument test runs a temporary zsh helper and /usr/bin/true.
The gate suite substitutes its child runner, so it does not recurse into a gate.
The memcap suite uses fake RSS samples and small disposable child processes.
"""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent / 'explore'))
sys.path.insert(0, str(Path(__file__).resolve().parent / 'viewer'))

SUITES = (
    'test_autostart_receipt',
    'test_replay_capture',
    'test_compare_unattended',
    'test_live_session',
    'test_rngcmp',
    'test_unattended_capture',
    'test_lane_lock',
    'test_capture_lanes',
    'test_census_layers',
    'test_viadriver',
    'test_waitwin',
    'test_startcapture',
    'test_brief',
    'test_gates',
    'test_trace_report',
    'test_recomp_snapshot',
    'test_gamelog_checks',
    'test_tracer_stamp',
    'test_census',
    'test_queueledger',
    'test_release_gate',
    'test_refill',  # the sixty-second pass, Loop 3769
    'test_ctx',  # the pass of 2026-11-14, second: a lane reads its own context
    'test_census_marks',  # the forty-seventh pass, Loop 3405
    'test_census_cites',  # the forty-eighth pass, Loop 3415
    'test_census_territory',  # item 4039: the territory's kinds and marks
    'test_samegame',  # the forty-ninth pass, Loop 3435
    'test_poolshot',  # the fiftieth pass, Loop 3469/3451
    'test_retire',  # the fifty-first pass, Loop 3486 (tools/, the script's own directory)
    'test_killguard',  # the forty-seventh pass, Loop 3392/3394
    'test_reap_agents',
    'test_memcap',
    'test_handover',  # the pass of 2026-11-13, DECISIONS 131 §8
    'test_book',  # the pass of 2026-11-26, DECISIONS 159: a booking touches one item
    'test_isolated_tests',
    'test_runtime_policy',
    'test_lab_demo',
    'test_branch_experiment',
    'test_captures_union',
    'test_seams',
    'test_standing',
    'test_tranche',
    'test_lanewait',
    'test_waitrun',
    'test_mutate',
    'test_repin',
    'test_cant_reach_pins',  # the steering pass of 2026-11-15: --write keeps the table's comments
    'test_sweep_gen',
    'test_unitaddr',
    'test_unitat',
    'test_filerun',
    'test_split',
    'test_runwiring',
    'test_births',
    'test_recomp_flags',
    'test_hooks_stubs',
    'test_hover_hook',  # item 3494
    'test_issue_queue',  # item 3504
    'test_hold_hook',  # item 3502
    'test_suite_timings',
    'test_suite_groups',  # item 4216: the release suite's groups and the audit's comparison
    'test_buildat',
    'test_emu_receipts',
    'test_emu_env',
    'test_offline_tests',
    'test_archive_census',
    'test_rubric',  # the forty-eighth pass's scorecard
    'test_frame_cut',  # item 3977
    'test_nations_census',  # Loop 3972: the census's --check
    'test_paths',  # item 4008: the cited source paths, checked and moved
    'test_shape',  # item 4008: the structure track's six counts
    'test_disk',  # the steering pass of 2026-11-12: the gate's disk steps
    'test_repin_packets',  # the pass of 2026-11-19: the packets' PINNED from a print walk
    'test_archive_compress',  # the pass of 2026-11-23, third: one tree at a time, and the scan (DECISIONS 156 §6)
)

# The lab's suites (`docs/DECISIONS.md` entry 38: merged whole, its ledger
# not scanned) and the probes that need an install or a bottle: listed by
# name so that a new `test_*.py` under tools/explore is either run by the
# gate or named here with its reason — three of the last two passes' own
# suites (sweep_gen, runwiring, births) ran in no gate for a day (parked
# 1872, the thirtieth pass).
NOT_RUN = {
    'test_allocation_selection': 'the lab', 'test_autostart_abi': 'the lab',
    'test_bounded_call': 'the lab', 'test_capsule': 'the lab', 'test_congestion_probe': 'the lab',
    'test_coord_table_probe': 'the lab', 'test_frame_snapshot': 'needs a bottle',
    'test_frame_snapshot_stub': 'the lab', 'test_frame_state': 'the lab', 'test_hook_stub': 'the lab',
    'test_jev_evidence_lab': 'the lab', 'test_lobby_focus': 'needs a window',
    'test_memory_inventory': 'the lab', 'test_memory_payload': 'the lab', 'test_order_capsule': 'the lab',
    'test_path_capsule': 'the lab', 'test_path_counterexample': 'the lab',
    'test_register_image_stub': 'the lab', 'test_restore_context': 'the lab',
    'test_restore_prefix': 'the lab', 'test_resume_frontier': 'the lab', 'test_search_census': 'the lab',
    'test_search_graph': 'the lab', 'test_startup_abi': 'the lab', 'test_startup_evidence': 'the lab',
    'test_thread_context_probe': 'the lab', 'test_typed_coverage_report': 'the lab',
    'test_typed_guy_bridge': 'the lab', 'test_typed_height_precision': 'the lab',
    'test_typed_leader_bridge': 'the lab', 'test_typed_leader_inventory': 'the lab',
    'test_typed_leader_projection': 'the lab', 'test_typed_logger_compare': 'the lab',
    'test_typed_logger_inventory': 'the lab', 'test_typed_oracle_compare': 'the lab',
    'test_typed_pin_coverage': 'the lab', 'test_typed_state': 'the lab', 'test_typed_world_bridge': 'the lab',
    'test_wow64_fault': 'the lab',
}

# These suites spend most of their time waiting for disposable child
# processes. Together they occupied about half of the single-worker run.
PROCESS_SUITES = {'test_isolated_tests', 'test_memcap', 'test_mutate', 'test_waitrun'}


def run_modules(names):
    timing = ['--durations', '10'] if hasattr(unittest.TestResult, 'addDuration') else []
    return unittest.main(module=None, argv=[sys.argv[0], *timing, *names], verbosity=2, exit=False).result


def run_two_workers():
    start = time.monotonic()
    with tempfile.TemporaryDirectory() as directory:
        summary = Path(directory) / 'worker.json'
        # Keep unittest's multi-line tracebacks intact. The foreground's
        # output is live and the process worker's complete log follows it.
        with tempfile.TemporaryFile(mode='w+', encoding='utf-8', errors='replace') as log:
            with subprocess.Popen([sys.executable, __file__, '--process-worker', str(summary)],
                                  stdout=log, stderr=subprocess.STDOUT) as worker:
                try:
                    other = run_modules([name for name in SUITES if name not in PROCESS_SUITES])
                    code = worker.wait()
                finally:
                    if worker.poll() is None:
                        worker.terminate()
                    worker.wait()
                    log.seek(0)
                    shutil.copyfileobj(log, sys.stderr)
        if code:
            return code
        counts = json.loads(summary.read_text())
        print(f"Offline tests: {other.testsRun + counts['ran']} ran in {time.monotonic() - start:.2f}s "
              f"with two workers, {len(other.skipped) + counts['skipped']} skipped", flush=True)
        return int(not other.wasSuccessful())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--jobs', type=int, choices=(1, 2), default=1)
    parser.add_argument('--process-worker', type=Path, help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.process_worker:
        result = run_modules([name for name in SUITES if name in PROCESS_SUITES])
        with args.process_worker.open('x') as summary:
            json.dump(dict(ran=result.testsRun, skipped=len(result.skipped)), summary)
        return int(not result.wasSuccessful())
    if args.jobs == 2:
        return run_two_workers()
    return int(not run_modules(SUITES).wasSuccessful())


if __name__ == '__main__':
    sys.exit(main())
