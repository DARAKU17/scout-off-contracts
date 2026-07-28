#!/usr/bin/env bash
# ScoutChain — generate TypeScript bindings for all contracts
# Usage: ./scripts/generate-bindings.sh [testnet|mainnet]
# Requires .env.contracts to exist (written by deploy.sh)
set -euo pipefail

# Pin the stellar-cli version to ensure reproducible bindings.
# Keep in sync with: docs/CONTRIBUTING.md and .github/workflows/contract-ci.yml
REQUIRED_STELLAR_CLI_VERSION="25.2.0"

actual_version=$(stellar --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || true)
if [[ "$actual_version" != "$REQUIRED_STELLAR_CLI_VERSION" ]]; then
  echo "ERROR: stellar-cli version mismatch."
  echo "       Required: $REQUIRED_STELLAR_CLI_VERSION"
  echo "       Found:    ${actual_version:-<not installed>}"
  echo ""
  echo "Install the correct version:"
  echo "  curl -sSL https://raw.githubusercontent.com/stellar/stellar-cli/v${REQUIRED_STELLAR_CLI_VERSION}/install.sh | bash"
  echo ""
  echo "See docs/CONTRIBUTING.md for setup instructions."
  exit 1
fi

NETWORK="${1:-testnet}"
# shellcheck disable=SC1091
source .env.contracts

CONTRACTS=(registration verification progress scout_access)

declare -A IDS=(
  [registration]="$REGISTRATION_CONTRACT_ID"
  [verification]="$VERIFICATION_CONTRACT_ID"
  [progress]="$PROGRESS_CONTRACT_ID"
  [scout_access]="$SCOUT_ACCESS_CONTRACT_ID"
)

for name in "${CONTRACTS[@]}"; do
  id="${IDS[$name]}"
  out="bindings/${name}"

  echo "==> Generating TypeScript bindings for $name ($id)..."
  stellar contract bindings typescript \
    --contract-id "$id" \
    --network "$NETWORK" \
    --output-dir "$out" \
    --overwrite

  echo "    Written to $out/"
done

# Extract function lists from CONTRACT_REFERENCE.md and inject into each bindings README
echo ""
echo "==> Injecting function lists into bindings READMEs..."

extract_function_list() {
  local contract_name="$1"
  local doc_file="docs/CONTRACT_REFERENCE.md"

  # Find the contract section and extract function signatures until next ## section
  awk -v contract="$contract_name" '
    BEGIN { in_section=0; in_functions=0; }
    /^## / {
      if ($2 == contract) {
        in_section=1;
      } else if (in_section) {
        exit;
      }
    }
    in_section && /^### Functions/ {
      in_functions=1;
      next;
    }
    in_section && in_functions && /^#### `/ {
      # Extract function signature
      match($0, /`([^`]+)`/, sig);
      getline; # skip blank line
      getline; # get description line or table start

      # Collect description until we hit a table or blank line
      desc = "";
      while (length($0) > 0 && !/^\|/) {
        if (desc != "") desc = desc " ";
        desc = desc $0;
        getline;
      }

      # If we got no description, provide a default
      if (desc == "") {
        desc = "See CONTRACT_REFERENCE.md for details";
      }

      # Print formatted entry
      print "- `" sig[1] "` — " desc;
    }
  ' "$doc_file"
}

# Build the auto-generated section for a given contract name and function list.
# Output is written to a temp file so awk can `system("cat ...")` it without
# needing to pass a multi-line variable through shell argument quoting.
write_auto_section() {
  local name="$1"
  local function_list="$2"
  local tmpfile="$3"
  printf '%s\n' \
    "<!-- AUTO-GENERATED FUNCTION LIST BEGIN - DO NOT EDIT MANUALLY -->" \
    "" \
    "## Functions" \
    "" \
    "The following functions are available in this contract. For complete documentation including parameters, return types, authorization requirements, and examples, see [CONTRACT_REFERENCE.md](../../docs/CONTRACT_REFERENCE.md#${name})." \
    "" \
    "$function_list" \
    "" \
    "<!-- AUTO-GENERATED FUNCTION LIST END -->" > "$tmpfile"
}

for name in "${CONTRACTS[@]}"; do
  readme="bindings/${name}/README.md"

  if [[ ! -f "$readme" ]]; then
    echo "    WARNING: $readme not found, skipping function list injection"
    continue
  fi

  # Extract function list
  function_list=$(extract_function_list "$name")

  if [[ -z "$function_list" ]]; then
    echo "    WARNING: No functions extracted for $name, skipping"
    continue
  fi

  tmpfile="$(mktemp)"
  write_auto_section "$name" "$function_list" "$tmpfile"

  # Check if auto-generated section already exists
  if grep -q "<!-- AUTO-GENERATED FUNCTION LIST BEGIN" "$readme"; then
    # Replace existing section using awk (idempotent — same output every run)
    awk -v tmpfile="$tmpfile" '
      /<!-- AUTO-GENERATED FUNCTION LIST BEGIN/ {
        system("cat " tmpfile);
        in_auto=1;
        next;
      }
      /<!-- AUTO-GENERATED FUNCTION LIST END -->/ {
        in_auto=0;
        next;
      }
      !in_auto { print; }
    ' "$readme" > "${readme}.tmp"
    mv "${readme}.tmp" "$readme"
    echo "    Updated function list in $readme"
  else
    # First time: ensure the README ends with exactly one newline, then append
    # the section (no extra blank line so a second run is idempotent).
    sed -i -e '$a\' "$readme"   # ensure trailing newline
    printf '\n' >> "$readme"    # one blank separator line
    cat "$tmpfile" >> "$readme"
    echo "    Added function list to $readme"
  fi

  rm -f "$tmpfile"
done

echo ""
echo "==> All bindings generated. Publish or link them into backend/frontend."
