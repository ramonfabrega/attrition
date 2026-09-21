"""Pure selection policy for a future bounded payload collector.

Consumes validated inventory rows; does not read memory or resolve pointers.
Heap addresses must come from the same event. This is not replay closure.
"""
from collections import defaultdict


def select(rows, *, main, observer, stack, roots, budget):
    """Return exact candidate spans or refuse. rows use the inventory v1 layout."""
    if budget <= 0 or not roots:
        raise ValueError('positive budget and explicit roots required')
    low, high = stack
    if not 0 <= low < high <= 2**32:
        raise ValueError('invalid excluded stack')
    by_allocation = defaultdict(list)
    for r in rows:
        base, allocation, _, size, state, protect, kind = r
        if not 0 <= base < base + size <= 2**32:
            raise ValueError('invalid range extent')
        eligible = (state == 0x1000 and not protect & 0x100
                    and protect & 0xff in (2, 4, 8)
                    and allocation != observer
                    and not (base < high and base + size > low)
                    and (kind == 0x20000 or (kind == 0x1000000 and allocation == main)))
        if eligible:
            by_allocation[allocation].append((base, size))
    if main not in by_allocation:
        raise ValueError('main image has no eligible data')
    chosen = {main}
    resolved = {}
    for name, address in roots.items():
        matches = [r for r in rows if r[0] <= address < r[0] + r[3]]
        if len(matches) != 1:
            raise ValueError(f'{name}: absent or ambiguous root')
        r = matches[0]
        if (r[0], r[3]) not in by_allocation.get(r[1], ()):
            raise ValueError(f'{name}: root is outside eligible data')
        chosen.add(r[1]); resolved[name] = r[1]
    spans = sorted((base, size, allocation) for allocation in chosen
                   for base, size in by_allocation[allocation])
    total = 0
    previous_end = 0
    for base, size, _ in spans:
        if base < previous_end:
            raise ValueError('overlapping selected spans')
        previous_end = base + size
        total += size
        if total > budget:
            raise ValueError(f'candidate exceeds {budget}-byte cap')
    return {'bytes': total, 'budget': budget, 'root_allocations': resolved,
            'allocations': sorted(chosen),
            'spans': [{'base': b, 'size': s, 'allocation': a} for b, s, a in spans],
            'closure_established': False}
