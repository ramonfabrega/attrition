"""Exercise sampling failures without launching a memory-heavy workload."""
import os
from pathlib import Path
import subprocess
import tempfile
import time
import unittest


MEMCAP = Path(__file__).resolve().parents[1] / "memcap.sh"


# The subprocess timeouts here are generous on purpose (parked 645, the
# thirteenth pass): at 8 s the suite timed out under a sibling lane's gate at
# load average 6.6, before any cargo step, and passed alone in 2.9 s. A gate
# that goes red on the box's load is a gate whose red means less, and two
# lanes and a commander gate on this box. What the tight timeout was also
# checking — that the wrapper kills its child when sampling fails — is now an
# elapsed-time assertion against a child that would otherwise run a minute.
TIMEOUT = 50
CHILD_SECONDS = 60
KILLED_WITHIN = 40


class SamplingFailureTests(unittest.TestCase):
    def run_guard(self, fake_ps, expected_code=125, child_must_die=False):
        with tempfile.TemporaryDirectory(prefix="attrition-memcap-") as directory:
            root = Path(directory)
            ps = root / "ps"
            ps.write_text("#!/bin/sh\n" + fake_ps)
            ps.chmod(0o755)
            env = dict(os.environ, PATH=f"{root}:{os.environ['PATH']}",
                       MEMCAP_TEST_STATE=str(root / "sampled"))
            started = time.monotonic()
            result = subprocess.run(
                ["zsh", str(MEMCAP), "20", "sleep", str(CHILD_SECONDS)],
                env=env, capture_output=True, text=True, timeout=TIMEOUT,
            )
            elapsed = time.monotonic() - started
            self.assertEqual(result.returncode, expected_code, result.stderr)
            if child_must_die:
                # The child sleeps a minute; a wrapper that stopped it
                # returns well inside that, whatever the box's load.
                self.assertLess(elapsed, KILLED_WITHIN,
                                f"the wrapper returned after {elapsed:.1f} s: the "
                                f"{CHILD_SECONDS} s child was not killed")
            return result.stderr

    def test_wrapper_sets_its_actual_cap_on_the_child(self):
        with tempfile.TemporaryDirectory(prefix="attrition-memcap-marker-") as directory:
            root = Path(directory)
            ps = root / "ps"
            ps.write_text("#!/bin/sh\necho 1024\n")
            ps.chmod(0o755)
            env = dict(os.environ, PATH=f"{root}:{os.environ['PATH']}",
                       RON_TEST_MEMCAP_GIB="999")
            result = subprocess.run(
                ["zsh", str(MEMCAP), "20", "sh", "-c", 'printf "%s" "$RON_TEST_MEMCAP_GIB"'],
                env=env, capture_output=True, text=True, timeout=TIMEOUT,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout, "20")

    def test_denied_sampling_refuses_launch(self):
        self.assertIn("refusing to launch", self.run_guard("exit 1\n"))

    def test_invalid_sample_refuses_launch(self):
        self.assertIn("invalid RSS", self.run_guard("echo unavailable\n"))

    def test_over_cap_sample_stops_live_command(self):
        message = self.run_guard(
            'if [ -e "$MEMCAP_TEST_STATE" ]; then echo 22020096; exit 0; fi\n'
            ': > "$MEMCAP_TEST_STATE"\n'
            'echo 1024\n', expected_code=137, child_must_die=True,
        )
        self.assertIn("over the 20 GiB ceiling", message)

    def test_sampling_loss_stops_live_command(self):
        # The preflight succeeds, then the first live sample fails. The
        # elapsed check is what says the minute-long child died with it.
        message = self.run_guard(
            'if [ -e "$MEMCAP_TEST_STATE" ]; then exit 1; fi\n'
            ': > "$MEMCAP_TEST_STATE"\n'
            'echo 1024\n', child_must_die=True,
        )
        self.assertIn("while the command is live", message)


if __name__ == "__main__":
    unittest.main()
