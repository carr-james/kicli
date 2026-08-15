#!/usr/bin/env bash
# PreToolUse hook: deny Edit/Write on the frozen surface.
# Input: hook JSON on stdin (tool_input.file_path). Exit 2 blocks the call
# and feeds stderr back to the model.
set -euo pipefail

list="$(dirname "$0")/frozen-paths.txt"
[ -f "$list" ] || exit 0

path="$(python3 -c 'import json,sys; print(json.load(sys.stdin).get("tool_input",{}).get("file_path",""))' 2>/dev/null || true)"
[ -n "$path" ] || exit 0

# Normalise to a repo-relative comparison.
case "$path" in
  /*) rel="${path#"$(git rev-parse --show-toplevel 2>/dev/null || pwd)/"}" ;;
  *)  rel="$path" ;;
esac

while IFS= read -r frozen; do
  case "$frozen" in ''|'#'*) continue ;; esac
  if [ "$rel" = "$frozen" ] || [ "${path%"$frozen"}" != "$path" ]; then
    echo "Frozen surface: $frozen — changes require a ruling; park as BLOCKED." >&2
    exit 2
  fi
done < "$list"
exit 0
