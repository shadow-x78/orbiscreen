# Branch Protection Policy

This document describes the branch protection policy applied to
`shadow-x78/orbiscreen:main` and `shadow-x78/orbiscreen:dev`.

## Policy

The policy is sent verbatim to the GitHub REST API via:

```bash
gh api -X PUT /repos/shadow-x78/orbiscreen/branches/main/protection \
    --input policy.json
```

### Settings

| Setting | Value | Reason |
|---------|-------|--------|
| `required_status_checks.strict` | `true` | Branches must be up-to-date before merging |
| `required_status_checks.contexts[]` | `["workspace"]` | CI must be green |
| `enforce_admins` | `true` | Even maintainers obey the rules |
| `required_pull_request_reviews.required_approving_review_count` | `1` | At least one approval |
| `required_pull_request_reviews.dismiss_stale_reviews` | `true` | Stale approvals are discarded |
| `required_linear_history` | `true` | No merge commits on protected branches; squash only |
| `allow_force_pushes` | `false` | Force pushes are forbidden |
| `allow_deletions` | `false` | `main` / `dev` cannot be deleted |
| `block_creations` | `false` | Anyone can create feature branches |
| `required_conversation_resolution` | `true` | All PR comments must be resolved |
| `lock_branch` | `false` | (default; do not lock) |
| `allow_fork_syncing` | `false` | Forks stay independent |

The top-level toggles are **boolean literals** in the JSON body — GitHub
rejects `"enforce_admins": {"enabled": true}` with a 422 validation error.

### `restrictions` field

For **personal** repositories (this one), the value must be
`"restrictions": null`. The field is structurally required by the schema
but GitHub rejects:

- Omitting it → `422 "restrictions" wasn't supplied`
- Any object with `users[]` / `teams[]` arrays → `422 Only organization
  repositories can have users and team restrictions`

The accepted form is `"restrictions": null` — GitHub silently drops the
push-restriction, which is exactly what a personal repo needs. For
**organization** repos, populate `users[]` / `teams[]` / `apps[]`.

### Required Status Check: `workspace`

The single required check is the `workspace` job from
`.github/workflows/ci.yml`. The Android workflow's `android` job is
reported but not required (the Android client is not productionised
yet).

The `Verify dependency licenses` job (`cargo deny check`) is also
reported but **not required** — it sometimes fails on the
unmaintained `derivative` transitive dep (RUSTSEC-2024-0388) which
arrives through `evdi v0.8.0`. See `docs/TROUBLESHOOTING.md` →
"CI Action: `Run cargo-deny`" for the full resolution path.

## Verification

```bash
gh api /repos/shadow-x78/orbiscreen/branches/main/protection | head -20
gh api /repos/shadow-x78/orbiscreen/branches/dev/protection   | head -20
```

Both should return the same JSON shape (the workspace + dev branches
share an identical policy).

## Why These Rules

- **`strict = true`** prevents racing a fast-forward PR against a fix
  that landed after the PR was opened.
- **`enforce_admins = true`** prevents the single owner from bypassing
  the rules when under pressure to ship quickly.
- **`required_approving_review_count = 1`** is the minimum for a
  project with a single owner; it enforces at least one second pair of
  eyes even when the owner is reviewing their own work via a fresh
  account.
- **`required_linear_history = true`** keeps `git log --first-parent`
  clean for release notes generation.
- **`allow_force_pushes = false`** protects already-tagged releases
  from being rewritten.
- **`allow_deletions = false`** prevents accidentally losing `main`
  or `dev` when the branch protection UI is reset.

## Solo-Dev Relax-and-Restore

Because the repo is single-owner, **the only reviewer cannot be the
author of the PR**. To merge your own PRs, use the relax-and-restore
pattern:

```bash
# 1. Apply the strict policy (default)
gh api -X PUT /repos/shadow-x78/orbiscreen/branches/<branch>/protection \
    --input policy.json

# 2. To merge your own PR: temporarily relax the two blocking rules
gh api -X PATCH \
    /repos/shadow-x78/orbiscreen/branches/<branch>/protection/required_pull_request_reviews \
    -f required_approving_review_count=0
gh api -X PUT /repos/shadow-x78/orbiscreen/branches/<branch>/protection \
    --input policy-relaxed.json   # enforce_admins: false

# 3. Merge the PR
gh pr merge <N> --squash --admin --delete-branch

# 4. Restore the strict policy
gh api -X PUT /repos/shadow-x78/orbiscreen/branches/<branch>/protection \
    --input policy.json
```

Once a second GitHub account exists to act as a reviewer, this dance
is no longer needed — the default `1 approval` requirement can be
satisfied by the second account.

## Re-applying After Repo Transfer or Rename

The policy is keyed to the branch name (`main`, `dev`). After a
transfer:

```bash
gh api -X PUT /repos/<new-owner>/orbiscreen/branches/main/protection \
    --input policy.json
gh api -X PUT /repos/<new-owner>/orbiscreen/branches/dev/protection \
    --input policy.json
```

## Troubleshooting

See `TROUBLESHOOTING.md` for the full troubleshooting table:
- Apply branch protection — `"enabled"` rejected as not boolean
- Apply branch protection — `restrictions` rejected on personal repo
- Apply branch protection — `"restrictions" wasn't supplied`
- PR merge blocked — `enforce_admins` and self-approval conflict
- Push rejected — `protected branch hook declined`
