"""Process-local allocator policy for test children; no host configuration edits."""
import os
import sys


def bounded_test(name):
    """Measured repeated walks; other isolated tests keep the system policy."""
    return (name == 'diff::coverage::every_parsed_field_is_compared_by_the_instrument_or_pinned'
            or name.startswith('diff::third_pair::'))


def child_environment(environ=None, *, platform=None, test_name=None, dispatcher=False):
    env = dict(os.environ if environ is None else environ)
    mode = env.get('RON_TEST_ALLOCATOR', 'auto')
    if mode not in ('auto', 'bounded', 'system'):
        raise ValueError('RON_TEST_ALLOCATOR must be auto, bounded or system')
    bounded = mode == 'bounded' or (mode == 'auto' and not dispatcher
                                   and (test_name is None or bounded_test(test_name)))
    if (sys.platform if platform is None else platform) == 'darwin' and bounded:
        # Apple's allocator can retain gigabytes of already-freed large blocks.
        # Reclaim them promptly for competing suites. Preserve explicit overrides;
        # this does not change type isolation, zeroing or corruption checks.
        env.setdefault('MallocLargeCache', '0')
    return env


def allocator_policy(env):
    return {'platform': sys.platform,
            'requested': env.get('RON_TEST_ALLOCATOR', 'auto'),
            'MallocLargeCache': env.get('MallocLargeCache')}
