#!/usr/bin/env bash
# Orbiscreen — apply branch protection policy (GPL-3.0-or-later)
# https://github.com/shadow-x78/orbiscreen
set -euo pipefail

REPO="shadow-x78/orbiscreen"
BRANCH="main"
POLICY="$(dirname "$0")/branch-protection.json"

echo "==> Applying branch protection to ${REPO}:${BRANCH}"

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    echo "    Using gh CLI"
    if ! gh api \
        -X PUT \
        -H "Accept: application/vnd.github+json" \
        "/repos/${REPO}/branches/${BRANCH}/protection" \
        --input "${POLICY}"; then
        echo
        echo "Hint: if GitHub rejected the policy with 'Only organization" >&2
        echo "      repositories can have users and team restrictions', make" >&2
        echo "      sure this is a personal repo and remove the 'restrictions'" >&2
        echo "      key from ${POLICY}." >&2
        exit 1
    fi
else
    echo "    Using curl + GitHub token from \$GH_TOKEN or \$GITHUB_TOKEN"
    TOKEN="${GH_TOKEN:-${GITHUB_TOKEN:-}}"
    if [[ -z "${TOKEN}" ]]; then
        echo "ERROR: gh is not authenticated and neither GH_TOKEN nor GITHUB_TOKEN is set." >&2
        echo "Hint:  export GH_TOKEN=<a-personal-access-token-with-repo-scope>" >&2
        exit 1
    fi
    if ! curl -fsSL \
        -X PUT \
        -H "Authorization: token ${TOKEN}" \
        -H "Accept: application/vnd.github+json" \
        "https://api.github.com/repos/${REPO}/branches/${BRANCH}/protection" \
        --data @"${POLICY}"; then
        echo
        echo "Hint: if GitHub rejected the policy with 'Only organization" >&2
        echo "      repositories can have users and team restrictions', make" >&2
        echo "      sure this is a personal repo and remove the 'restrictions'" >&2
        echo "      key from ${POLICY}." >&2
        exit 1
    fi
    echo
fi

echo "==> Verifying"
if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    gh api "/repos/${REPO}/branches/${BRANCH}/protection" | grep -E '"strict"|"enabled"|"required_approving_review_count"|"dismiss_stale_reviews"'
else
    echo "    (skipped verification — install gh CLI or re-run with \$GH_TOKEN to verify)"
fi

echo "==> Done."