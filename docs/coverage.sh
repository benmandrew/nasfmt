#!/bin/sh
# Generate docs/coverage.svg from cargo-tarpaulin --engine llvm output.
# Requires: cargo-tarpaulin  (cargo install cargo-tarpaulin)
# Usage: sh docs/coverage.sh
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TMPL="$SCRIPT_DIR/coverage.svg.tmpl"
OUT="$SCRIPT_DIR/coverage.svg"

echo "Running cargo tarpaulin..."
if ! TARPAULIN_OUT=$(cargo tarpaulin --engine llvm); then
    exit 1
fi

PCT_FULL=$(printf '%s\n' "$TARPAULIN_OUT" \
    | grep '% coverage' \
    | grep -oE '^[0-9]+\.[0-9]+' \
    || true)

if [ -z "$PCT_FULL" ]; then
    printf '%s\n' "$TARPAULIN_OUT" >&2
    echo "error: could not parse coverage percentage from tarpaulin output" >&2
    exit 1
fi

PCT=$(printf '%.0f' "$PCT_FULL")

if   [ "$PCT" -ge 90 ]; then COLOR="#4c1"
elif [ "$PCT" -ge 80 ]; then COLOR="#a4a61d"
elif [ "$PCT" -ge 60 ]; then COLOR="#dfb317"
else                          COLOR="#e05d44"
fi

sed -e "s/{{COLOR}}/$COLOR/g" -e "s/{{PCT}}/$PCT/g" "$TMPL" > "$OUT"

echo "Coverage: ${PCT_FULL}%  →  badge ${PCT}% ($COLOR)  →  $OUT"
