# Branch Protection Policy

This document defines the required branch protection rules for `main`. The
GitHub UI banner — *"Your main branch isn't protected"* — is resolved by
applying the policy below once the repository is created.

## Policy

Apply via the GitHub REST API after `git push` lands the first commit:

```bash
gh api -X PUT \
    /repos/shadow-x78/orbiscreen/branches/main/protection \
    --input admin/branch-protection.json
```

Or, with `curl` and a personal access token (`PAT`):

```bash
curl -X PUT \
    -H "Authorization: token ${PAT}" \
    -H "Accept: application/vnd.github+json" \
    https://api.github.com/repos/shadow-x78/orbiscreen/branches/main/protection \
    --data @admin/branch-protection.json
```

## Required Status Checks

The CI workflow at `.github/workflows/ci.yml` exposes a single required
check named **`workspace`**. The CI workflow must be green before any PR
to `main` can merge. The check:

- Runs `cargo fmt --all -- --check`
- Runs `cargo clippy --workspace --all-targets --locked -- -D warnings`
- Runs `cargo build --workspace --locked`
- Runs `cargo test --workspace --locked`

The Android workflow at `.github/workflows/android.yml` exposes a check
named **`android`** that builds the debug APK. Treat it as recommended
(not required) until the Android client is productionised.

## Rules Summary

| Setting | Value | Reason |
|---------|-------|--------|
| `required_status_checks.strict` | `true` | Branches must be up-to-date before merging |
| `required_status_checks.contexts[]` | `["workspace"]` | CI must be green |
| `enforce_admins` | `true` | Even maintainers obey the rules |
| `required_pull_request_reviews.required_approving_review_count` | `1` | At least one approval |
| `required_pull_request_reviews.dismiss_stale_reviews` | `true` | Stale approvals are discarded |
| `restrictions.users[]` | `[]` | No extra collaborators beyond the team |
| `restrictions.teams[]` | `[]` | (n/a — single-owner project) |
| `required_linear_history` | `true` | No merge commits on `main`; squash only |
| `allow_force_pushes` | `false` | Force pushes are forbidden |
| `allow_deletions` | `false` | `main` cannot be deleted |
| `block_creations` | `false` | Anyone can create feature branches |
| `required_conversation_resolution` | `true` | All PR comments must be resolved |
| `lock_branch` | `false` | (default; do not lock) |
| `allow_fork_syncing` | `false` | Forks stay independent |

Note: the top-level boolean toggles (`enforce_admins`, `required_linear_history`, `allow_force_pushes`, etc.) are **boolean literals**, not `{enabled: true}` wrappers. The GitHub REST API only accepts the literal form.

## Why These Rules

- **`strict = true`** prevents racing a fast-forward PR against a fix that
  landed after the PR was opened.
- **`enforce_admins = true`** prevents the single owner from bypassing the
  rules when under pressure to ship quickly.
- **`required_approving_review_count = 1`** is the minimum for a project
  with a single owner; it enforces at least one second pair of eyes even
  when the owner is reviewing their own work via a fresh account.
- **`required_linear_history = true`** keeps `git log --first-parent` clean
  for release notes generation.
- **`allow_force_pushes = false`** protects already-tagged releases from
  being rewritten.
- **`allow_deletions = false`** prevents accidentally losing `main` when
  the branch protection UI is reset.

## Re-applying After Repo Transfer or Rename

The policy is keyed to the branch name (`main`). After a transfer:

```bash
gh api -X PUT \
    /repos/<new-owner>/orbiscreen/branches/main/protection \
    --input admin/branch-protection.json
```

## Verifying

After applying:

```bash
gh api /repos/shadow-x78/orbiscreen/branches/main/protection
```

Expected response shape (top-level toggles are **boolean literals**, not objects):

```json
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["workspace"]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": true,
    "required_approving_review_count": 1
  },
  "required_linear_history": true,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "required_conversation_resolution": true,
  "block_creations": false,
  "lock_branch": false,
  "allow_fork_syncing": false
}
```

The presence of the `restrictions` key in the response means the policy
was applied successfully (it appears even with empty `users[]` / `teams[]`
arrays because GitHub always echoes the setting back).

### Debugging Failed Applies

If `gh api` returns `422 Invalid request`, the JSON body of the error
explains the schema violation. The most common cause is wrapping a
top-level toggle (e.g. `enforce_admins`) in `{enabled: ...}` instead of
using the literal boolean — always flatten to `true` / `false`.