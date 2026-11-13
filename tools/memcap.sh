#!/bin/zsh
# memcap.sh <gib> <command…> — run a command under a sampled resident-memory
# ceiling, and kill it on the first sample over the limit.
#
# **Why this exists.** On 2026-09-04 a worker's `cargo test -p rondata
# --release` grew to 27.6 GB resident at 472 % CPU; swap filled, 14 MB of
# RAM was left, and the machine went down eight minutes after another
# session noticed. Every part of that chain worked except the last one: a
# message cannot save a machine that is already thrashing. The ceiling has
# to be mechanical, it has to be *outside* the process, and it has to fire
# without anyone reading anything.
#
# `ulimit -v` is not it: macOS does not enforce `RLIMIT_AS` for the
# mappings a Rust test binary makes, so a limit set in the shell is a limit
# the child ignores. What does work is watching `ps` and sending a signal,
# which is what this does — poll the command's own resident set, plus every
# descendant's, every two seconds, and `kill -9` them on the first sample
# over the ceiling. Two seconds is fast enough: the run above took minutes
# to climb, and a poll that samples growth of gigabytes per minute cannot
# miss it by more than a few hundred megabytes.
#
# It walks **descendants by parent id** rather than killing a process
# group, because the thing that grows is a test binary `cargo` spawned
# rather than `cargo` itself — and because a group kill from inside the
# group takes this script with it, which loses the exit status and the
# reason (that is what the first version of this file did, and the way it
# was found is the way every guard here is found: by making it fire).
#
# It counts the tree at the **larger of its resident set and its physical
# footprint** (`proc_pid_rusage`'s `ri_phys_footprint`, the number Activity
# Monitor and jetsam use), each summed over the tree. `ps rss` alone under-reads exactly when
# the guard matters: a page macOS has compressed or swapped is not resident,
# so a process whose memory the host is squeezing reads small while it still
# holds every byte. The fixture's child — 1,200 MiB of one repeated byte,
# idle, the cheapest page the compressor can take — was held for 120 s on a
# booking gate (gate-4192) while the resource budget's own `ps rss` sampler
# saw its whole test binary's tree peak at 1,159 MiB, against 1.8–2.4 GiB on
# every green gate; this script, reading the same `ps`, never fired. The
# footprint counts compressed pages, and a test that fakes a small resident
# set over a large footprint holds that door (`tools/explore/test_memcap.py`).
#
# `MEMCAP_TRACE=<file>` appends every sample — the seconds since launch, the
# tree, both measures, the counted total and the cap, in KiB — so a red that
# says "never fired" can say what it saw and when, and a gap in the times
# says the sampler itself stalled. It is off unless asked for: a gate's
# half-hour suite would otherwise write nine hundred lines to its log.
#
#   zsh tools/memcap.sh 8 cargo test -p rondata --release
#
# Exit status is the command's own, except when the ceiling fires: then it
# is 137 (128 + SIGKILL) and the reason is on stderr, so a gate that reads
# the exit code cannot mistake an OOM kill for a test failure.
# RSS sampling failure returns 125; the command is not left unmonitored.
set -u

if (( $# < 2 )); then
    print -u2 "usage: memcap.sh <gib> <command…>"
    exit 2
fi

# A missing process-sampling grant must not turn a guarded run into an
# unmonitored one that reports a fictitious zero-MiB peak.
probe_rss=$(ps -o rss= -p $$ 2>/dev/null) || {
    print -u2 "memcap: cannot sample process RSS; refusing to launch an unmonitored command"
    exit 125
}
probe_rss=${probe_rss//[[:space:]]/}
if [[ "$probe_rss" != <-> ]] || (( probe_rss <= 0 )); then
    print -u2 "memcap: invalid RSS sample; refusing to launch an unmonitored command"
    exit 125
fi

# `pid rss_kib footprint_kib` per live pid named; exits 1 if a live pid could
# not be read (a pid that has exited is skipped, as `ps` skips it).
footprints() {
    python3 -I -c '
import ctypes, errno, sys
lib = ctypes.CDLL("/usr/lib/libproc.dylib", use_errno=True)
info = (ctypes.c_uint64 * 12)()  # rusage_info_v0: uuid[16], then ten u64
bad = 0
for pid in sys.argv[1:]:
    if lib.proc_pid_rusage(int(pid), 0, ctypes.byref(info)) == 0:
        print(pid, info[8] // 1024, info[9] // 1024)  # ri_resident_size, ri_phys_footprint
    elif ctypes.get_errno() != errno.ESRCH:
        bad = 1
sys.exit(bad)
' "$@" 2>/dev/null
}

probe_fp=$(footprints $$) || {
    print -u2 "memcap: cannot sample process footprint; refusing to launch an unmonitored command"
    exit 125
}
if [[ "${probe_fp##* }" != <-> ]] || (( ${probe_fp##* } <= 0 )); then
    print -u2 "memcap: invalid footprint sample; refusing to launch an unmonitored command"
    exit 125
fi

cap_gib=$1; shift
cap_kb=$(( cap_gib * 1024 * 1024 ))

zmodload zsh/datetime
launched=$EPOCHREALTIME
trace() {
    [[ -n ${MEMCAP_TRACE:-} ]] || return 0
    printf '%7.1fs %s\n' $(( EPOCHREALTIME - launched )) "$*" >> $MEMCAP_TRACE
}

# Every descendant of $1, depth first, one pid per line.
descendants() {
    local p=$1 kid
    for kid in ${(f)"$(pgrep -P $p 2>/dev/null)"}; do
        [[ -n $kid ]] || continue
        print $kid
        descendants $kid
    done
}

# Mark only the monitored child and its descendants, using this wrapper's
# actual cap rather than any inherited marker. This is a cooperative contract.
# `nice`: a capture starved under four lanes' suites (parked 1693, load 130
# on 16 cores with the game at 2 % CPU), so a guarded suite yields to it.
RON_TEST_MEMCAP_GIB="$cap_gib" nice -n 10 "$@" &
pid=$!
trace "launched pid $pid under a ${cap_gib} GiB cap ($cap_kb KiB): ${*//$'\n'/; }"

peak_kb=0
peak_one_kb=0
rung=0
while kill -0 $pid 2>/dev/null; do
    tree=($pid ${(f)"$(descendants $pid)"})
    rss_rows= fp_rows=
    if ! rss_rows=$(ps -o rss= -p ${(j:,:)tree} 2>/dev/null) \
        || ! fp_rows=$(footprints ${tree[@]}); then
        # A short-lived command may finish between kill -0 and ps. Otherwise
        # fail closed rather than let a live command run without monitoring.
        kill -0 $pid 2>/dev/null || break
        trace "sampling failed: ps [${rss_rows//$'\n'/; }] footprint [${fp_rows//$'\n'/; }]"
        print -u2 "memcap: RSS sampling failed while the command is live — stopping its process tree"
        kill -9 ${tree[@]} 2>/dev/null
        wait $pid 2>/dev/null
        exit 125
    fi
    # Each measure's sum over the tree and its largest single process; the
    # counted figure is the larger of the two measures, for both.
    sample=$(print -rl -- "$rss_rows" -- "$fp_rows" | awk '
        $1 == "--" { fp = 1; next }
        !fp && NF >= 1 { rs += $1; if ($1 > rm) rm = $1 }
        fp && NF >= 3 { fs += $3; if ($3 > fm) fm = $3 }
        END { print (rs > fs ? rs : fs) + 0, (rm > fm ? rm : fm) + 0, rs + 0, fs + 0 }')
    sample=(${=sample})
    rss_kb=$sample[1]
    one_kb=$sample[2]
    trace "tree ${(j:,:)tree} rss $sample[3] footprint $sample[4] counted $rss_kb cap $cap_kb KiB"
    (( rss_kb > peak_kb )) && peak_kb=$rss_kb
    (( one_kb > peak_one_kb )) && peak_one_kb=$one_kb
    # A rung per gibibyte, so the log says *when* it climbed: interleaved
    # with the test names a suite prints, that names the hungry test
    # without a profiler.
    if (( rss_kb / 1024 / 1024 > rung )); then
        rung=$(( rss_kb / 1024 / 1024 ))
        print -u2 "memcap: past ${rung} GiB (largest single process $(( one_kb / 1024 )) MiB)"
    fi
    if (( rss_kb > cap_kb )); then
        print -u2 "memcap: $1 and its children reached $(( rss_kb / 1024 )) MiB, over the ${cap_gib} GiB ceiling — killing them"
        trace "over the cap: killing ${(j:,:)tree}"
        kill -9 ${tree[@]} 2>/dev/null
        wait $pid 2>/dev/null
        exit 137
    fi
    sleep 2
done

# `status` is read-only in zsh — naming the variable that way made the
# success path die with "read-only variable" and swallow both the peak
# line and the exit code. `rc` it is.
wait $pid
rc=$?
trace "the command exited $rc"
print -u2 "memcap: peak $(( peak_kb / 1024 )) MiB across the tree, $(( peak_one_kb / 1024 )) MiB in the largest single process, of a ${cap_gib} GiB ceiling"
exit $rc
