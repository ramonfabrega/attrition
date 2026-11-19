#!/bin/sh
# Run from the repository root under the guard or gate reservation.
set -e
python3 tools/queueledger.py
exec python3 tools/shape.py --check
