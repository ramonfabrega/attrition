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

cap_gib=$1; shift
cap_kb=$(( cap_gib * 1024 * 1024 ))

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
RON_TEST_MEMCAP_GIB="$cap_gib" "$@" &
pid=$!

peak_kb=0
peak_one_kb=0
rung=0
while kill -0 $pid 2>/dev/null; do
    tree=($pid ${(f)"$(descendants $pid)"})
    if ! rss_rows=$(ps -o rss= -p ${(j:,:)tree} 2>/dev/null); then
        # A short-lived command may finish between kill -0 and ps. Otherwise
        # fail closed rather than let a live command run without monitoring.
        kill -0 $pid 2>/dev/null || break
        print -u2 "memcap: RSS sampling failed while the command is live — stopping its process tree"
        kill -9 ${tree[@]} 2>/dev/null
        wait $pid 2>/dev/null
        exit 125
    fi
    sample=$(print -r -- "$rss_rows" | awk '{s+=$1; if ($1>m) m=$1} END {print s+0, m+0}')
    rss_kb=${sample%% *}
    one_kb=${sample##* }
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
print -u2 "memcap: peak $(( peak_kb / 1024 )) MiB across the tree, $(( peak_one_kb / 1024 )) MiB in the largest single process, of a ${cap_gib} GiB ceiling"
exit $rc
