#!/usr/bin/env bash
set -Eeuo pipefail

REPO="${1:-taberoajorge/loopforge}"
BRANCH="${2:-main}"

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "missing dependency: $1" >&2; exit 1; }
}

need gh
need jq

echo "target repo: $REPO"
echo "target branch: $BRANCH"
echo

visibility=$(gh api "repos/$REPO" --jq '.visibility')
echo "visibility: $visibility"
if [[ "$visibility" != "public" ]]; then
  cat <<MSG
The branch ruleset, secret scanning, push protection and code scanning default
setup require the repository to be public or a GitHub Pro plan. Re run this
script after switching visibility with:

  gh repo edit $REPO --visibility public --accept-visibility-change-consequences

MSG
  exit 0
fi

echo "enabling private vulnerability reporting"
gh api --method PUT "repos/$REPO/private-vulnerability-reporting" || true

echo "enabling secret scanning and push protection"
gh api --method PATCH "repos/$REPO" --input - <<'JSON'
{
  "security_and_analysis": {
    "secret_scanning": { "status": "enabled" },
    "secret_scanning_push_protection": { "status": "enabled" },
    "secret_scanning_non_provider_patterns": { "status": "enabled" },
    "secret_scanning_validity_checks": { "status": "enabled" },
    "dependabot_security_updates": { "status": "enabled" }
  }
}
JSON

echo "enforcing SHA pinning for GitHub Actions"
gh api --method PUT "repos/$REPO/actions/permissions" \
  -F enabled=true -F allowed_actions=all -F sha_pinning_required=true

echo "creating or updating branch ruleset for $BRANCH"
existing=$(gh api "repos/$REPO/rulesets" --jq ".[] | select(.name==\"protect-$BRANCH\") | .id" || true)

payload=$(cat <<JSON
{
  "name": "protect-$BRANCH",
  "target": "branch",
  "enforcement": "active",
  "bypass_actors": [],
  "conditions": {
    "ref_name": {
      "include": ["refs/heads/$BRANCH"],
      "exclude": []
    }
  },
  "rules": [
    { "type": "deletion" },
    { "type": "non_fast_forward" },
    { "type": "required_linear_history" },
    {
      "type": "pull_request",
      "parameters": {
        "required_approving_review_count": 1,
        "dismiss_stale_reviews_on_push": true,
        "require_code_owner_review": true,
        "require_last_push_approval": true,
        "required_review_thread_resolution": true
      }
    },
    {
      "type": "required_status_checks",
      "parameters": {
        "strict_required_status_checks_policy": true,
        "required_status_checks": [
          { "context": "typecheck" },
          { "context": "lint" },
          { "context": "rust-tests" }
        ]
      }
    }
  ]
}
JSON
)

if [[ -n "$existing" ]]; then
  echo "updating ruleset $existing"
  echo "$payload" | gh api --method PUT "repos/$REPO/rulesets/$existing" --input -
else
  echo "creating ruleset"
  echo "$payload" | gh api --method POST "repos/$REPO/rulesets" --input -
fi

echo "enabling CodeQL default setup (requires public repo or GHAS)"
gh api --method PATCH "repos/$REPO/code-scanning/default-setup" \
  -F state=configured -F query_suite=default || true

echo "all security settings applied"
