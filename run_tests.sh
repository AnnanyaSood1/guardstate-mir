#!/usr/bin/env bash
# Build the stable crates and run the golden-file MIR-text tests.
set -euo pipefail
cd "$(dirname "$0")"
cargo build --quiet
BIN=./target/debug/gsm-text
TD=crates/gsm-text/tests_mir
FB=$TD/forbidden.txt
pass=0; fail=0
run() {  # $1 = test file, $2 = detector
  local b exp got
  b=$(basename "$1" .mir); exp="$TD/expected/$b.txt"
  got=$("$BIN" "$1" --forbidden "$FB" --detector "$2")
  if diff -u "$exp" <(printf '%s\n' "$got") >/dev/null; then
    echo "PASS  $b"; pass=$((pass+1))
  else
    echo "FAIL  $b"; diff -u "$exp" <(printf '%s\n' "$got") || true; fail=$((fail+1))
  fi
}
for t in "$TD"/t1_* "$TD"/t2_* "$TD"/t3_* "$TD"/t4_* "$TD"/t5_* "$TD"/t6_* "$TD"/t7_*; do run "$t" block-in-atomic; done
for t in "$TD"/t8_* "$TD"/t9_*; do run "$t" any-guard; done
echo "-----------------------------"
echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
