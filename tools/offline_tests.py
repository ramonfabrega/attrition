#!/usr/bin/env python3
"""Run the reviewed offline acquisition and gate tests without an install.

Use explicit module names: new experimental tests are not discovered implicitly.
The cleanup suite mocks live-session operations and uses authored temporary
profiles; its launch-argument test runs a temporary zsh helper and /usr/bin/true.
The gate suite substitutes its child runner, so it does not recurse into a gate.
The memcap suite uses fake RSS samples and small disposable child processes.
"""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent / 'explore'))
sys.path.insert(0, str(Path(__file__).resolve().parent / 'viewer'))

SUITES = (
    'test_autostart_receipt',
    'test_compare_unattended',
    'test_live_session',
    'test_rngcmp',
    'test_unattended_capture',
    'test_lane_lock',
    'test_capture_lanes',
    'test_census_layers',
    'test_viadriver',
    'test_waitwin',
    'test_runqueue',
    'test_startcapture',
    'test_brief',
    'test_gamelog_checks',
    'test_tracer_stamp',
    'test_census',
    'test_queueledger',
    'test_release_gate',
    'test_memcap',
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
    'test_sweep_gen',
    'test_unitaddr',
    'test_unitat',
    'test_runwiring',
    'test_births',
    'test_recomp_flags',
    'test_hooks_stubs',
    'test_offline_tests',
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

if __name__ == '__main__':
    unittest.main(module=None, argv=[sys.argv[0], *SUITES], verbosity=2)
