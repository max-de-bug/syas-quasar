#!/usr/bin/env bash
# Compare build / test wall-clock time: Anchor SYAS vs Quasar SYAS.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
QUASAR_PROJECT="$(dirname "$SCRIPT_DIR")"
ANCHOR_PROJECT="$(dirname "$QUASAR_PROJECT")/solana-yield-adapter-standard"
RESULTS_DIR="${QUASAR_PROJECT}/benchmark-results"
TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
REPORT="${RESULTS_DIR}/benchmark-${TIMESTAMP}.txt"

mkdir -p "$RESULTS_DIR"

log() { echo "$@" | tee -a "$REPORT"; }

time_phase() {
  local label="$1"
  shift
  log ""
  log "=== ${label} ==="
  log "Command: $*"
  local start end elapsed
  start=$(date +%s.%N)
  if "$@"; then
    end=$(date +%s.%N)
    elapsed=$(python3 -c "print(f'{float($end) - float($start):.2f}')")
    log "OK  ${elapsed}s"
    echo "$elapsed"
  else
    log "FAILED"
    echo "FAIL"
  fi
}

count_tests() {
  local dir="$1"
  local pattern="$2"
  grep -r "$pattern" --include="*.rs" "$dir" 2>/dev/null | grep -v "mod tests" | wc -l || echo 0
}

log "SYAS Framework Benchmark — ${TIMESTAMP}"
log "Anchor project: ${ANCHOR_PROJECT}"
log "Quasar project: ${QUASAR_PROJECT}"

# Ensure Agave 3.1.10 toolchain is active for Quasar builds.
AGAVE_311="${HOME}/.local/share/solana/install/releases/3.1.10/solana-release/bin"
if [ -d "$AGAVE_311" ]; then
  export PATH="${AGAVE_311}:${PATH}"
  log "Agave 3.1.10 toolchain active: ${AGAVE_311}"
else
  log "WARN: Agave 3.1.10 not found at ${AGAVE_311} - Quasar builds may fail"
fi

# --- Anchor ---
log ""
log "########## ANCHOR ##########"

if [[ -d "$ANCHOR_PROJECT" ]]; then
  A_BUILD=$(time_phase "Anchor: build" bash -c "cd '$ANCHOR_PROJECT' && npm run build" 2>&1 | tail -1 || echo FAIL)
  A_TEST=$(time_phase "Anchor: test execution only" bash -c "cd '$ANCHOR_PROJECT' && npm test" 2>&1 | tail -1 || echo FAIL)
  A_PROGRAMS=$(find "$ANCHOR_PROJECT/programs" -maxdepth 1 -type d | wc -l)
  A_TESTS=$(count_tests "$ANCHOR_PROJECT/programs" "#\[test\]")
  log "Anchor build: ${A_BUILD}s"
  log "Anchor local tests: ${A_TEST}s"
  log ""
  log "Anchor full fork (build+clone+deploy+20 TS tests):"
  log "  cd '$ANCHOR_PROJECT' && time npm run test:fork"
else
  log "Anchor project not found at ${ANCHOR_PROJECT}"
  A_BUILD="N/A"
  A_TEST="N/A"
  A_PROGRAMS=0
  A_TESTS=0
fi

# --- Quasar ---
log ""
log "########## QUASAR ##########"

Q_BUILD="N/A"
Q_RUST_TEST="N/A"
Q_PROGRAMS=0
Q_RUST_TESTS=0

if [[ -x "${QUASAR_PROJECT}/scripts/build-quasar.sh" ]]; then
  Q_BUILD=$(time_phase "Quasar: build" bash -c "cd '$QUASAR_PROJECT' && npm run build" 2>&1 | tail -1 || echo FAIL)
  log "Quasar build: ${Q_BUILD}s"
fi

if [[ -f "${QUASAR_PROJECT}/Cargo.toml" ]]; then
  Q_RUST_TEST=$(time_phase "Quasar: cargo test" bash -c "cd '$QUASAR_PROJECT' && CARGO_TARGET_DIR='${QUASAR_PROJECT}/target' cargo test 2>&1" | tail -1 || echo FAIL)
  log "Quasar cargo test: ${Q_RUST_TEST}s"
fi

Q_PROGRAMS=$(find "${QUASAR_PROJECT}/programs" -maxdepth 1 -type d | wc -l)
Q_RUST_TESTS=$(count_tests "${QUASAR_PROJECT}/programs" "#\[test\]")

log ""
log "Quasar full fork — run manually:"
log "  cd '$QUASAR_PROJECT' && time npm run test:fork"

# --- Summary ---
log ""
log "########## SUMMARY ##########"
log ""
printf "%-30s %-15s %-15s\n" "Metric" "Anchor" "Quasar"
printf "%-30s %-15s %-15s\n" "------" "------" "------"
printf "%-30s %-15s %-15s\n" "Build time (all programs)" "${A_BUILD}s" "${Q_BUILD}s"
printf "%-30s %-15s %-15s\n" "Test time (local)" "${A_TEST}s" "${Q_RUST_TEST}s"
printf "%-30s %-15s %-15s\n" "Program count" "$A_PROGRAMS" "$Q_PROGRAMS"
printf "%-30s %-15s %-15s\n" "Rust unit-test count" "$A_TESTS" "$Q_RUST_TESTS"
log ""
log "Report saved: ${REPORT}"
