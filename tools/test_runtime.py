"""Process-local allocator policy for test children; no host configuration edits."""
import os
import sys


def child_environment(environ=None, *, platform=None):
    env = dict(os.environ if environ is None else environ)
    mode = env.get('RON_TEST_ALLOCATOR', 'bounded')
    if mode not in ('bounded', 'system'):
        raise ValueError('RON_TEST_ALLOCATOR must be bounded or system')
    if (sys.platform if platform is None else platform) == 'darwin' and mode == 'bounded':
        # Apple's allocator can retain gigabytes of already-freed large blocks.
        # Reclaim them promptly for competing suites. Preserve explicit overrides;
        # this does not change type isolation, zeroing or corruption checks.
        env.setdefault('MallocLargeCache', '0')
    return env


def allocator_policy(env):
    return {'platform': sys.platform,
            'requested': env.get('RON_TEST_ALLOCATOR', 'bounded'),
            'MallocLargeCache': env.get('MallocLargeCache')}
