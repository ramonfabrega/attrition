#!/bin/zsh
# backup.sh [sync|restore|pending|ls [prefix]] — the off-machine copy of
# everything the diffs are pinned against and git does not hold.
#
# Until 2026-09-04 the capture corpus had exactly one copy, on a disk with
# 50 GB free and no Time Machine destination. `rondata::diff`'s floors are
# assertions against these files, and the long human-driven runs (run53's
# 24,000 frames) cannot be re-created by script — so a dead SSD would have
# turned the score into a number in a commit message. This mirrors three
# machine-local trees into a private R2 bucket:
#
#   ~/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations   the corpus
#       (Logs/, and the inis the lane runs with)      -> s3://ron-data/rise-of-nations/
#   ~/Documents/My Games/Rise of Nations              recordings and saves
#                                                      -> s3://ron-data/my-games/
#   ~/ghidra-projects                                  the analysed project and
#       the decompile export (hours to rebuild)        -> s3://ron-data/ghidra-projects/
#
# The flow does not change: captures land on disk, tools read from disk, and
# this pushes what is new — `aws s3 sync` compares size and mtime, so a run
# with nothing new uploads nothing. `restore` is the other direction, for a
# new machine. `pending` is the same walk with `--dryrun`: what would go,
# sending nothing, which is the question to ask before booking the wall
# clock. Neither sync nor restore deletes on the far side: a local deletion
# is not propagated unless `RON_BACKUP_DELETE=1`, because a backup that
# mirrors a mistake is not one.
#
# **Interrupting is safe, and is the expected way to stop.** Every mode
# compares per file and sends only what is missing, so Ctrl-C and a re-run
# resumes; a partial run deletes nothing on either side.
#
# Credentials come from passage at run time and live nowhere on disk:
# `tokens/cloudflare/personal/r2/ron-data/{access-key-id,secret-access-key}`,
# a token scoped to this one bucket, Object Read & Write. `AWS_ACCESS_KEY_ID`
# and `AWS_SECRET_ACCESS_KEY` already in the environment win, for a machine
# without passage. The account id is not a secret; it is the endpoint.
#
# Excluded: the live `gamelog.txt` (archive.sh renames it into the corpus
# when a run ends), `.DS_Store`, `__pycache__`, and Ghidra's lock files.
set -e
set -o pipefail

mode=${1:-sync}
ACCOUNT=${RON_R2_ACCOUNT:-2deff21d2e459d1c5997c4fe57a208e2}
BUCKET=${RON_R2_BUCKET:-ron-data}
export AWS_ENDPOINT_URL="https://$ACCOUNT.r2.cloudflarestorage.com"
export AWS_DEFAULT_REGION=auto
# The aws CLI's newer default checksums are not what R2 speaks; these are the
# values Cloudflare documents for the S3 API.
export AWS_REQUEST_CHECKSUM_CALCULATION=when_required
export AWS_RESPONSE_CHECKSUM_VALIDATION=when_required

if [ -z "$AWS_ACCESS_KEY_ID" ] || [ -z "$AWS_SECRET_ACCESS_KEY" ]; then
  P=tokens/cloudflare/personal/r2/ron-data
  AWS_ACCESS_KEY_ID=$(passage show "$P/access-key-id")
  AWS_SECRET_ACCESS_KEY=$(passage show "$P/secret-access-key")
  export AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY
fi
[ -n "$AWS_ACCESS_KEY_ID" ] && [ -n "$AWS_SECRET_ACCESS_KEY" ] || {
  echo "no R2 credentials: passage $P, or AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY" >&2
  exit 1
}

# local path, bucket prefix, extra excludes
trees=(
  "$HOME/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations|rise-of-nations|--exclude Logs/gamelog.txt"
  "$HOME/Documents/My Games/Rise of Nations|my-games|"
  "$HOME/ghidra-projects|ghidra-projects|--exclude *.lock --exclude *.ulock"
)
common=(--exclude '.DS_Store' --exclude '*/.DS_Store' --exclude '*__pycache__/*')
[ "${RON_BACKUP_DELETE:-0}" = 1 ] && common+=(--delete)

# **The progress counter is for a person; the transcript is for an agent.**
# `aws`'s progress line redraws with a carriage return, which a terminal
# renders as one live counter and a captured log renders as a wall of
# half-lines. So it is on when stdout is a TTY and off otherwise: 22 GB with
# no counter is what "stuck" looks like, and the same 22 GB in a tool result
# is what "unreadable" looks like. Neither audience has to ask for a flag.
progress=(--no-progress)
[ -t 1 ] && progress=()

# For the headers: the tree under its own name rather than eleven characters
# of this machine's home directory.
short() { echo "${1/#$HOME/~}"; }

case $mode in
  ls)
    # With no argument the whole bucket, which is the ghidra export's tens of
    # thousands of files plus the corpus and answers "is the mirror ~22 GB".
    # With one it is a prefix — `ls rise-of-nations` is the count that moves
    # by one archive and one trace per capture, and is the one a capture lane
    # actually checks.
    aws s3 ls "s3://$BUCKET/${2:-}" --summarize --human-readable --recursive | tail -3
    ;;
  sync|restore|pending)
    started=$SECONDS
    for t in "${trees[@]}"; do
      local_dir=${t%%|*}; rest=${t#*|}; prefix=${rest%%|*}; extra=${rest#*|}
      args=("${common[@]}" ${=extra} "${progress[@]}")
      [ "$mode" = pending ] && args+=(--dryrun --no-progress)
      if [ "$mode" = restore ]; then
        mkdir -p "$local_dir"
        src="s3://$BUCKET/$prefix"; dst=.
      else
        [ -d "$local_dir" ] || { echo "skip (not here): $(short "$local_dir")" >&2; continue; }
        src=.; dst="s3://$BUCKET/$prefix"
      fi
      echo "== $mode $prefix  ($(short "$local_dir"))"
      tree_start=$SECONDS
      # **One tree's failure must not skip the others.** R2 hands back a
      # transient `ServiceUnavailable` or `InvalidPart` on a large multipart
      # upload often enough that the first run of 2026-09-04 aborted under
      # `set -e` with two of the three trees never started. A sync is
      # idempotent and resumable, so the useful behaviour is to carry on and
      # report at the end; the exit code still says something went wrong.
      #
      # The `cd` is what makes the per-file lines readable: `aws` prints the
      # local side relative to the working directory, and run from the repo
      # that is eleven `../` and then the whole path again, on every line.
      if ! ( cd "$local_dir" && aws s3 sync "$src" "$dst" "${args[@]}" ); then
        echo "!! $mode failed for $prefix — re-run, it resumes" >&2
        failed=1
      fi
      echo "   $prefix: $((SECONDS - tree_start))s"
    done
    if [ "${failed:-0}" = 1 ]; then
      echo "$mode finished with failures after $((SECONDS - started))s — re-run \`backup.sh $mode\`, then \`ls\`" >&2
      exit 1
    fi
    echo "$mode done in $((SECONDS - started))s"
    ;;
  *)
    echo "usage: backup.sh [sync|restore|pending|ls [prefix]]" >&2; exit 2
    ;;
esac
