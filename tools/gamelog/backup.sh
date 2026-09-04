#!/bin/zsh
# backup.sh [sync|restore|ls] — the off-machine copy of everything the diffs
# are pinned against and git does not hold.
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
# new machine. Neither deletes on the far side: a local deletion is not
# propagated unless `RON_BACKUP_DELETE=1`, because a backup that mirrors a
# mistake is not one.
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

case $mode in
  ls)
    aws s3 ls "s3://$BUCKET/" --summarize --human-readable --recursive | tail -3
    ;;
  sync|restore)
    for t in "${trees[@]}"; do
      local_dir=${t%%|*}; rest=${t#*|}; prefix=${rest%%|*}; extra=${rest#*|}
      if [ "$mode" = sync ]; then
        [ -d "$local_dir" ] || { echo "skip (not here): $local_dir" >&2; continue; }
        src="$local_dir"; dst="s3://$BUCKET/$prefix"
      else
        src="s3://$BUCKET/$prefix"; dst="$local_dir"; mkdir -p "$local_dir"
      fi
      echo "== $mode: $src -> $dst"
      aws s3 sync "$src" "$dst" "${common[@]}" ${=extra} --no-progress
    done
    echo "$mode done"
    ;;
  *)
    echo "usage: backup.sh [sync|restore|ls]" >&2; exit 2
    ;;
esac
