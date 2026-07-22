#!/usr/bin/env bash
# Orbiscreen — apply branch protection policy (GPL-3.0-or-later)
# https://github.com/shadow-x78/orbiscreen
#
# The policy in `branch-protection.json` is sent verbatim to the GitHub
# REST API endpoint:
#   PUT /repos/{owner}/{repo}/branches/main/protection
#
# Note for personal repositories (this repo is personal, owned by
# `shadow-x78`):
#
#   The schema requires the `restrictions` key to be present, but
#   rejects `{users: [], teams: []}` with
#     422 Validation Failed: Only organization repositories can have
#     users and team restrictions
#   The accepted form is **`"restrictions": null`** — GitHub then omits
#   any push-restriction from the policy, which is exactly what a
#   personal repo needs.
#
# The policy will not apply successfully until the `workspace` status
# check exists in `.github/workflows/ci.yml` — otherwise the API rejects
# the `contexts[]` array as referring to a non-existent check.
set -euo pipefail

REPO="shadow-x78/orbiscreen"
BRANCH="main"
POLICY="$(dirname "$0")/branch-protection.json"

echo "==> Applying branch protection to ${REPO}:${BRANCH}"

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    echo "    Using gh CLI"
    gh api \
        -X PUT \
        -H "Accept: application/vnd.github+json" \
        "/repos/${REPO}/branches/${BRANCH}/protection" \
        --input "${POLICY}"
else
    echo "    Using curl + GitHub token from \$GH_TOKEN or \$GITHUB_TOKEN"
    TOKEN="${GH_TOKEN:-${GITHUB_TOKEN:-}}"
    if [[ -z "${TOKEN}" ]]; then
        echo "ERROR: gh is not authenticated and neither GH_TOKEN nor GITHUB_TOKEN is set." >&2
        echo "Hint:  export GH_TOKEN=<a-personal-access-token-with-repo-scope>" >&2
        exit 1
    fi
    curl -fsSL \
        -X PUT \
        -H "Authorization: token ${TOKEN}" \
        -H "Accept: application/vnd.github+json" \
        "https://api.github.com/repos/${REPO}/branches/${BRANCH}/protection" \
        --data @"${POLICY}"
    echo
fi

echo
echo "==> Verifying"
gh api "/repos/${REPO}/branches/${BRANCH}/protection" 2>/dev/null | python3 -c '
import json, sys
d = json.load(sys.stdin)
print("  strict:                          ", d["required_status_checks"]["strict"])
print("  contexts:                        ", d["required_status_checks"]["contexts"])
print("  enforce_admins:                  ", d["enforce_admins"]["enabled"])
print("  required_approving_review_count: ", d["required_pull_request_reviews"]["required_approving_review_count"])
print("  dismiss_stale_reviews:           ", d["required_pull_request_reviews"]["dismiss_stale_reviews"])
print("  required_linear_history:         ", d["required_linear_history"]["enabled"])
print("  allow_force_pushes:              ", d["allow_force_pushes"]["enabled"])
print("  allow_deletions:                 ", d["allow_deletions"]["enabled"])
print("  block_creations:                 ", d["block_creations"]["enabled"])
print("  required_conversation_resolution:", d["required_conversation_resolution"]["enabled"])
print("  lock_branch:                     ", d["lock_branch"]["enabled"])
print("  allow_fork_syncing:              ", d["allow_fork_syncing"]["enabled"])
'

echo
echo "==> Done."