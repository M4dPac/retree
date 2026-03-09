#!/bin/bash
set -e

PASS="\033[32m✓\033[0m"
FAIL="\033[31m✗\033[0m"
BOLD="\033[1m"
NC="\033[0m"

run() {
	local label="$1"
	shift
	printf "  %-20s" "$label"
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
run "check" cargo check
run "fmt" cargo fmt --all -- --check
run "clippy" cargo clippy --all-targets --all-features -- -D warnings
run "tests" cargo test

echo ""
echo -e "  ${BOLD}All checks passed${NC}"
echo ""
