#!/usr/bin/env bash
# Copyright (C) 2026 rezky_nightky
# SPDX-License-Identifier: GPL-3.0-only

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_ROOT}"

if [ "$#" -ne 1 ]; then
    echo "Usage: ./scripts/version-to.sh vMAJOR.MINOR.PATCH" >&2
    exit 1
fi

TARGET_TAG="$1"
if [[ ! "${TARGET_TAG}" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9.-]+)?$ ]]; then
    echo "Error: version must match vMAJOR.MINOR.PATCH" >&2
    exit 1
fi

TARGET_VERSION="${TARGET_TAG#v}"
CURRENT_VERSION="$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')"
CURRENT_TAG="v${CURRENT_VERSION}"

if [ "${CURRENT_VERSION}" = "${TARGET_VERSION}" ]; then
    echo "Already at ${TARGET_TAG}"
    exit 0
fi

sed -i -E "0,/^version = \"${CURRENT_VERSION}\"/s//version = \"${TARGET_VERSION}\"/" Cargo.toml

if [ -f Cargo.lock ]; then
    sed -i -E "/^name = \"zenritme\"$/{n;s/^version = \"${CURRENT_VERSION}\"/version = \"${TARGET_VERSION}\"/;}" Cargo.lock
fi

for file in README.md TRADEMARK.md docs/RELEASE_CHECKLIST.md; do
    [ -f "${file}" ] || continue
    sed -i -E "s/${CURRENT_TAG}/${TARGET_TAG}/g; s/${CURRENT_VERSION}/${TARGET_VERSION}/g" "${file}"
done

echo "Version bumped: ${CURRENT_TAG} -> ${TARGET_TAG}"
