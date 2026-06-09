#!/usr/bin/env bash
# Build all SYAS programs with Quasar (cargo build-sbf via quasar CLI).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
QUASAR_ROOT="${QUASAR_ROOT:-$(dirname "$PROJECT_DIR")/quasar}"

# Verify Agave 3.1.10 is active (required for platform-tools v1.52)
AGAVE_311="${HOME}/.local/share/solana/install/releases/3.1.10/solana-release/bin"
if [ -d "$AGAVE_311" ]; then
  export PATH="${AGAVE_311}:${PATH}"
else
  echo "WARN: Agave 3.1.10 not found at ${AGAVE_311}"
  echo "  Install with: agave-install init 3.1.10"
  echo "  Falling back to active release..."
fi

CURRENT_AGAVE=$(solana --version 2>/dev/null || echo "unknown")
echo "Using Solana/Agave: $CURRENT_AGAVE"

if ! command -v quasar >/dev/null 2>&1; then
  echo "ERROR: quasar CLI not found. Install from ${QUASAR_ROOT}/cli"
  exit 1
fi

PROGRAMS=(
  adapter-registry
  adapter-kamino
  adapter-marginfi
  adapter-jupiter
  adapter-maple
  adapter-drift
  adapter-template
  yield-dispatcher
)

mkdir -p "${PROJECT_DIR}/target/deploy"
mkdir -p "${PROJECT_DIR}/target/idl"

echo "Building ${#PROGRAMS[@]} programs with Quasar..."

for name in "${PROGRAMS[@]}"; do
  dir="${PROJECT_DIR}/programs/${name}"
  if [[ ! -f "${dir}/Quasar.toml" ]]; then
    echo "SKIP ${name} — no Quasar.toml (not ported yet)"
    continue
  fi
  echo "--- quasar build: ${name} ---"
  (cd "$dir" && quasar build)
  so_name="${name//-/_}"
  if [[ -f "${dir}/target/deploy/${so_name}.so" ]]; then
    cp "${dir}/target/deploy/${so_name}.so" "${PROJECT_DIR}/target/deploy/"
  fi
  # Preserve existing keypairs (don't overwrite with quasar-generated ones)
  if [[ -f "${dir}/target/deploy/${so_name}-keypair.json" ]] && [[ ! -f "${PROJECT_DIR}/target/deploy/${so_name}-keypair.json" ]]; then
    cp "${dir}/target/deploy/${so_name}-keypair.json" "${PROJECT_DIR}/target/deploy/"
  fi
  # Copy Quasar IDL to target/idl/ so TS test runner can find it
  if [[ -f "${dir}/target/idl/${so_name}.json" ]]; then
    cp "${dir}/target/idl/${so_name}.json" "${PROJECT_DIR}/target/idl/"
  fi
done

echo "Done. Artifacts in target/deploy/ and target/idl/"
