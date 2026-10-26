#!/usr/bin/env bash
# Saves the Happy Hare objects of a real printer as a test fixture (read-only: one HTTP GET, nothing is sent to
# Klipper). `crates/core/tests/fixtures.rs` checks every file in that folder against the rules of the core.
# Look through the file before committing it: it can contain spool names and other personal values.
# Usage (repo root): scripts/capture-printer-dump.sh <moonraker-url> <name> [--force]
#   scripts/capture-printer-dump.sh http://printer.local:7125 happy-hare-4.1.0-single-gate-psf
set -euo pipefail

cd "$(dirname "$0")/.." || exit 1
[ $# -ge 2 ] || { sed -n '2,7p' "$0" | sed 's/^# \{0,1\}//'; exit 2; }
url=${1%/}
name=$2
force=${3:-}
target="crates/core/tests/fixtures/$name.json"

if [ -e "$target" ] && [ "$force" != "--force" ]; then
    echo "$target exists; pass --force to replace it" >&2
    exit 1
fi
mkdir -p "$(dirname "$target")"

response=$(curl --silent --show-error --fail --max-time 10 "$url/printer/objects/query?mmu&mmu_machine")
printf '%s' "$response" | python3 -I -c '
import json, sys
status = json.load(sys.stdin)["result"]["status"]
if "mmu" not in status:
    sys.exit("the printer reports no mmu object")
json.dump(status, sys.stdout, indent=2, sort_keys=True)
print()
' > "$target"
echo "saved $target ($(wc -l < "$target") lines); review it before committing"
