#!/usr/bin/env bash
set -euo pipefail

PASS=$'\033[32m✓\033[0m'
FAIL=$'\033[31m✗\033[0m'
BOLD=$'\033[1m'
NC=$'\033[0m'

TARGET_DIR="${CARGO_TARGET_DIR:-target}"
RUN_TMPDIR="$(mktemp -d "${TMPDIR:-/tmp}/retree-checks.XXXXXX")"

export TMPDIR="$RUN_TMPDIR"
export TMP="$RUN_TMPDIR"
export TEMP="$RUN_TMPDIR"

cleanup() {
  local exit_code=$?
  trap - EXIT

  rm -rf "$RUN_TMPDIR"
  rm -rf "$TARGET_DIR/tmp"

  # По умолчанию чистим только release, чтобы не терять debug-кэш.
  if [[ "${FULL_CLEAN:-0}" == "1" ]]; then
    cargo clean --quiet >/dev/null 2>&1 || true
  else
    cargo clean --release --quiet >/dev/null 2>&1 || true
  fi

  exit "$exit_code"
}

trap cleanup EXIT
trap 'exit 130' INT TERM

run() {
  local label="$1"
  shift

  printf "  %-28s" "$label"
  if output=$("$@" 2>&1); then
    echo -e "$PASS"
  else
    echo -e "$FAIL"
    echo ""
    echo -e "\033[31m$output\033[0m"
    exit 1
  fi
}

echo ""
echo -e "${BOLD}Running checks...${NC}"
echo ""

run "audit" cargo audit
run "deny" cargo deny check
run "check" cargo check
run "check --locked" cargo check --locked
run "fmt" cargo fmt --all -- --check

run "clippy lib+bins" cargo clippy --locked --lib --bins -- -D warnings -W clippy::unwrap_used
run "clippy tests+bench" cargo clippy --locked --tests --benches -- -D warnings
run "tests" cargo test --locked
run "tests tree_compat" cargo test --locked --features tree_compat
run "build --release --locked" cargo build --release --locked

echo ""
echo -e "  ${BOLD}All checks passed${NC}"
echo ""
