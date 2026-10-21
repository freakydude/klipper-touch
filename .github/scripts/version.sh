#!/bin/bash
# Computes the version from git tags. Releases are tagged vX.Y.0; every later commit adds one to the patch number.
#   version.sh            version of HEAD, e.g. 0.2.5 (main), 0.2.5-pr.147.3 (pull request), 0.2.5-dev.abc1234 (other)
#   version.sh minor      next release version after increasing the minor number, e.g. 0.3.0
#   version.sh major      next release version after increasing the major number, e.g. 1.0.0
#   version.sh last       tag of the last release, empty if there is none
set -euo pipefail

last_release=$(git tag --merged HEAD --list 'v*' | grep -E '^v[0-9]+\.[0-9]+\.0$' | sort -V | tail -n 1 || true)
if [ -n "$last_release" ]; then
  base="${last_release#v}"
  commits=$(git rev-list --count "$last_release..HEAD")
else
  base="0.1.0"
  commits=$(git rev-list --count HEAD)
fi
IFS=. read -r major minor _ <<< "$base"

case "${1:-}" in
  last) echo "$last_release"; exit 0 ;;
  minor) echo "$major.$((minor + 1)).0"; exit 0 ;;
  major) echo "$((major + 1)).0.0"; exit 0 ;;
  "") ;;
  *) echo "usage: $0 [last|minor|major]" >&2; exit 2 ;;
esac

version="$major.$minor.$commits"
case "${GITHUB_EVENT_NAME:-}:${GITHUB_REF:-}" in
  push:refs/heads/main) ;;
  pull_request:*) version="$version-pr.${PR_NUMBER:?}.${GITHUB_RUN_NUMBER:?}" ;;
  *) version="$version-dev.$(git rev-parse --short=7 HEAD)" ;;
esac
echo "$version"
