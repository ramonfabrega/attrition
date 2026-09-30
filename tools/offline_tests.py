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
)

if __name__ == '__main__':
    unittest.main(module=None, argv=[sys.argv[0], *SUITES], verbosity=2)
