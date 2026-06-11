#!/usr/bin/env bash
# =============================================================================
# run-fork-surfpool.sh — Run mainnet-fork tests via Surfpool (Quasar)
#
# Prerequisites:
#   1. Install Surfpool: curl -sL https://run.surfpool.run/ | bash
#   2. Set MAINNET_RPC_URL to a mainnet RPC endpoint (Helius, Triton, etc.)
#
# Usage:
#   bash scripts/run-fork-surfpool.sh
#   # or with a custom RPC:
#   export MAINNET_RPC_URL=https://mainnet.helius-rpc.com/?api-key=YOUR_KEY
#   bash scripts/run-fork-surfpool.sh
# =============================================================================
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
FIXTURE_DIR="${PROJECT_DIR}/tests/fixtures"

# Use --network mainnet by default; override with MAINNET_RPC_URL for custom RPC
NETWORK_OR_RPC="--network mainnet"
if [ -n "${MAINNET_RPC_URL:-}" ]; then
  NETWORK_OR_RPC="--rpc-url \"$MAINNET_RPC_URL\""
fi

if ! command -v surfpool &>/dev/null; then
  echo "ERROR: surfpool not found. Install: curl -sL https://run.surfpool.run/ | bash"
  exit 1
fi

echo "============================================"
echo "  Solana Yield Adapter Standard (Quasar)"
echo "  Mainnet-Fork Tests via Surfpool"
echo "============================================"

# 1. Prepare fork fixture accounts
echo ""
echo "[1/5] Preparing fork fixtures..."
bash "$SCRIPT_DIR/setup-fork-usdc-fixture.sh"
bash "$SCRIPT_DIR/setup-fork-syrup-usdc-fixture.sh"

# 2. Build programs (Quasar build)
echo ""
echo "[2/5] Building programs..."
cd "$PROJECT_DIR"
bash "$SCRIPT_DIR/build-quasar.sh"

# 3. Start Surfpool (mainnet fork with JIT account fetching)
echo ""
echo "[3/5] Starting Surfpool validator..."
surfpool stop 2>/dev/null || true
sleep 1
eval surfpool start \
  "$NETWORK_OR_RPC" \
  --snapshot "$FIXTURE_DIR/surfpool-snapshot.json" \
  --no-tui \
  --no-deploy \
  --legacy-anchor-compatibility \
  --ci \
  --daemon \
  --db :memory: 2>&1 | tail -5

echo "  Waiting for validator (up to 180s)..."
for i in $(seq 1 90); do
  if solana cluster-version -u http://127.0.0.1:8899 &>/dev/null; then
    echo "  Validator ready."
    break
  fi
  sleep 2
done
if ! solana cluster-version -u http://127.0.0.1:8899 &>/dev/null; then
  echo "ERROR: Validator failed to start within 180s"
  surfpool stop 2>/dev/null || true
  exit 1
fi

# 4. Deploy programs
echo ""
echo "[4/5] Deploying programs..."
for so in "$PROJECT_DIR/target/deploy"/*.so; do
  base=$(basename "$so" .so)
  keypair="$PROJECT_DIR/target/deploy/${base}-keypair.json"
  if [ -f "$keypair" ]; then
    echo "  Deploying $base..."
    solana program deploy "$so" \
      --program-id "$keypair" \
      -u http://127.0.0.1:8899
  else
    echo "  SKIP $base: missing keypair"
  fi
done

# 5. Run fork tests
echo ""
echo "[5/5] Running fork tests..."
MAINNET_FORK=1 npx ts-mocha \
  tests/registry.test.ts \
  tests/dispatcher.test.ts \
  tests/adapters/kamino.test.ts \
  tests/adapters/marginfi.test.ts \
  tests/adapters/jupiter.test.ts \
  tests/adapters/maple.test.ts \
  tests/adapters/drift.test.ts \
  --timeout 120000 --exit

echo ""
echo "============================================"
echo "  All mainnet-fork tests passed!"
echo "============================================"

# Cleanup
surfpool stop 2>/dev/null || true
