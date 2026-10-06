#!/usr/bin/env bash
# test_http_client_feature.sh
#
# Verifies the http-client feature gate:
#   (a) With http-client (default): builds, tests pass, reqwest 0.13 is present.
#   (b) Without http-client: builds, tests pass, and the dependency tree contains
#       no rustls-platform-verifier and no security-framework -- the crates that
#       emit Apple-framework linker flags and break cross-compilation to macOS.
#
# Usage:
#   ./tests/test_http_client_feature.sh [cross-target]
#
# Requires cargo and a Rust toolchain. Optionally pass a target triple to also
# check that target's dependency tree (e.g. aarch64-apple-darwin).

set -euo pipefail

CROSS_TARGET="${1:-}"

pass() { echo "PASS: $1"; }
fail() { echo "FAIL: $1"; exit 1; }

echo "(a) http-client on (default features)"

cargo check --quiet || fail "cargo check with default features failed"
pass "cargo check"

cargo test --quiet >/dev/null || fail "cargo test with default features failed"
pass "cargo test"

if cargo tree --quiet 2>/dev/null | grep -q "reqwest v0.13"; then
  pass "reqwest 0.13 present in dep tree"
else
  fail "reqwest 0.13 not found in dep tree (expected with http-client)"
fi

echo "(b) http-client off (--no-default-features)"

TARGET_FLAG=""
if [ -n "$CROSS_TARGET" ]; then
  TARGET_FLAG="--target $CROSS_TARGET"
  echo "    checking target: $CROSS_TARGET"
fi

cargo check --no-default-features --quiet $TARGET_FLAG || fail "cargo check without http-client failed"
pass "cargo check"

cargo test --no-default-features --quiet >/dev/null || fail "cargo test without http-client failed"
pass "cargo test"

TREE_OUT=$(cargo tree --no-default-features $TARGET_FLAG 2>/dev/null)

if echo "$TREE_OUT" | grep -q "reqwest v0.13"; then
  fail "reqwest 0.13 still in dep tree without http-client"
fi
pass "reqwest 0.13 absent from dep tree"

if echo "$TREE_OUT" | grep -q "rustls-platform-verifier"; then
  fail "rustls-platform-verifier still in dep tree without http-client"
fi
pass "rustls-platform-verifier absent from dep tree"

if echo "$TREE_OUT" | grep -q "security-framework"; then
  fail "security-framework still in dep tree without http-client"
fi
pass "security-framework absent from dep tree"

echo "All checks passed."
