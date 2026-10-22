#!/usr/bin/env python3
"""dumptime.py <rontrace.log> [<gamelog.txt>] [--json]: where a dumped frame's seconds go.

Reads the dump clock's records (`dumptime.h`, item 3483; `rontrace.cfg`'s
`dumptime=1|2`) and prints, a frame a row, the frame's seconds, its clock
state, the bytes the CRT handed `WriteFile` and the bytes of the gamelog's
block the frame's end dump wrote (` BEGIN FRAME f+1` to the next marker),
then the frame's seconds split by site, exclusive (a site's own time, its
timed callees' taken off):

    walk     GameLog::begin_frame, end_frame, full_dump, dump_units,
             dump_leaders — the walk and every object's `log_data`
    net      NetDaemon::process_all, which each walker opens with
    say      Log::say's three overloads: check_accept, the String copies
    printf   __stdio_common_vfwprintf: the format and the wide->narrow
             text-mode conversion into the stream's buffer
    putwc    fputwc: the indent, a space a call
    flush    fflush (and fclose, fputws if any ran)
    write    ucrtbase's WriteFile, inclusive: the syscall
    rest     the frame's seconds outside every timed site: the simulation,
             the render, the tracer

Ticks are `rdtsc`'s; seconds are QPC's, over the run's own ratio (the sum
of every frame clock's QPC ticks over its TSC ticks). The calibration line
is the probe's price: one timed call's TSC ticks over an untimed one's.
"""
import json
import re
import struct
import sys

K_INFO = 5
I_DUMPCLOCK, I_DUMPTIME, I_DUMPEXCL, I_DUMPSITE, I_DUMPCAL = 25, 26, 27, 28, 29
SITES = ['begin_frame', 'end_frame', 'full_dump', 'dump_units', 'dump_leaders',
         'say_int', 'say_ulong', 'say_float', 'process_all',
         'vfwprintf', 'fputwc', 'fflush', 'fclose', 'fputws', 'WriteFile']
GROUPS = {'walk': {0, 1, 2, 3, 4}, 'net': {8}, 'say': {5, 6, 7}, 'printf': {9}, 'putwc': {10},
          'flush': {11, 12, 13}, 'write': {14}}


def read(path):
    data = open(path, 'rb').read()
    out = {'sites': {}, 'cal': None, 'clock': {}, 'time': {}}
    for i in range(1, len(data) // 32):
        k, code, a, b, c, d, e, _frame = struct.unpack_from('<8I', data, i * 32)
        if k != K_INFO:
            continue
        if code == I_DUMPSITE:
            out['sites'][a] = {'va': b, 'refusal': c, 'kind': d, 'stub': e}
        elif code == I_DUMPCAL:
            out['cal'] = {'n': a, 'direct': b, 'stubbed': c, 'qpc_freq': d, 'tsc_per_64th': e}
        elif code == I_DUMPCLOCK:
            out['clock'][a] = {'qpc': b, 'tsc': c | d << 32, 'on': e & 0xff, 'leaks': (e >> 8) & 0xffff,
                               'depth': e >> 24}
        elif code == I_DUMPTIME:
            s = out['time'].setdefault(a >> 8, {}).setdefault(a & 0xff, {})
            s.update(calls=b, incl=c | d << 32, bytes_lo=e)
        elif code == I_DUMPEXCL:
            s = out['time'].setdefault(a >> 8, {}).setdefault(a & 0xff, {})
            s.update(excl=b | c << 32, bytes=s.get('bytes_lo', 0) | d << 32)
    return out


def blocks(path):
    """Bytes of each ` BEGIN FRAME N` block, to the next marker (or the end)."""
    sizes, last, start = {}, None, 0
    with open(path, 'rb') as f:
        data = f.read()
    for m in re.finditer(rb'^ ?BEGIN FRAME (\d+)\r?$', data, re.M):
        if last is not None:
            sizes[last] = m.start() - start
        last, start = int(m.group(1)), m.start()
    if last is not None:
        sizes[last] = len(data) - start
    return sizes


def main(argv):
    as_json = '--json' in argv
    argv = [a for a in argv if a != '--json']
    log = read(argv[0])
    sizes = blocks(argv[1]) if len(argv) > 1 else {}
    qpc = sum(c['qpc'] for c in log['clock'].values())
    tsc = sum(c['tsc'] for c in log['clock'].values())
    freq = (log['cal'] or {}).get('qpc_freq') or 10_000_000
    sec = (qpc / tsc / freq) if tsc else 0.0  # seconds a TSC tick
    rows = []
    for f in sorted(log['clock']):
        c, t = log['clock'][f], log['time'].get(f, {})
        split = {g: sum(t.get(i, {}).get('excl', 0) for i in ids) * sec for g, ids in GROUPS.items()}
        split['write'] = sum(t.get(i, {}).get('incl', 0) for i in GROUPS['write']) * sec
        split['rest'] = c['tsc'] * sec - sum(split.values())
        rows.append({'frame': f, 'on': c['on'], 'seconds': c['qpc'] / freq, 'leaks': c['leaks'],
                     'write_bytes': t.get(14, {}).get('bytes', 0), 'block': f + 1,
                     'block_bytes': sizes.get(f + 1), 'split': split,
                     'calls': {SITES[i]: v.get('calls', 0) for i, v in sorted(t.items()) if i < len(SITES)}})
    if as_json:
        json.dump({'sites': log['sites'], 'cal': log['cal'], 'sec_per_tick': sec, 'frames': rows}, sys.stdout,
                  indent=1)
        print()
        return 0
    for i, s in sorted(log['sites'].items()):
        name = SITES[i] if i < len(SITES) else f'site{i}'
        print(f'site {i:2} {name:13} va {s["va"]:#010x} kind {s["kind"]} '
              + ('installed' if not s['refusal'] else f'REFUSED {s["refusal"]}'))
    if log['cal']:
        c = log['cal']
        print(f'calibration: {c["n"]} calls, direct {c["direct"]} ticks, stubbed {c["stubbed"]} ticks: '
              f'{(c["stubbed"] - c["direct"]) / c["n"]:.1f} ticks ({(c["stubbed"] - c["direct"]) / c["n"] * sec * 1e9:.0f} ns) '
              f'a timed call; QPC {freq} Hz; {1 / sec / 1e6 if sec else 0:.1f} M ticks a second')
    cols = ['walk', 'net', 'say', 'printf', 'putwc', 'flush', 'write', 'rest']
    print(f'{"frame":>6} {"on":>2} {"seconds":>8} {"MB written":>10} {"block MB":>8} '
          + ' '.join(f'{c:>7}' for c in cols) + '   calls')
    for r in rows:
        if not r['calls'] and r['seconds'] < 0.5 and not r['block_bytes']:
            continue
        bb = f'{r["block_bytes"] / 1e6:8.3f}' if r['block_bytes'] else f'{"-":>8}'
        print(f'{r["frame"]:6} {r["on"]:2} {r["seconds"]:8.3f} {r["write_bytes"] / 1e6:10.3f} {bb} '
              + ' '.join(f'{r["split"][c]:7.3f}' for c in cols)
              + '   ' + ' '.join(f'{k}={v}' for k, v in r['calls'].items() if v))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
