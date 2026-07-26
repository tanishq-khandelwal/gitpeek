#!/bin/bash
# Builds a synthetic repo whose stashes look like real work — no real project data.
set -e
DIR="${1:-/tmp/lazystash-demo}"
G=/usr/bin/git
rm -rf "$DIR"; mkdir -p "$DIR"; cd "$DIR"
$G init -q --initial-branch=main .
$G config user.email dev@example.com
$G config user.name "Dev"
$G config core.autocrlf false

mkdir -p src/api src/ui
cat > src/api/client.ts <<'EOF'
export async function fetchUser(id: string) {
  const res = await fetch(`/api/users/${id}`);
  if (!res.ok) throw new Error("request failed");
  return res.json();
}
EOF
cat > src/ui/Button.tsx <<'EOF'
export function Button({ label, onClick }: Props) {
  return <button className="btn" onClick={onClick}>{label}</button>;
}
EOF
cat > src/cache.ts <<'EOF'
const store = new Map<string, string>();

export function get(key: string) {
  return store.get(key);
}
EOF
cat > README.md <<'EOF'
# example-app
EOF
$G add -A && $G commit -qm "initial commit"

stash() { # message, then edits already applied
  $G stash -q -u -m "$1"
}

# 1
cat > src/cache.ts <<'EOF'
const store = new Map<string, { value: string; expires: number }>();

export function get(key: string) {
  const hit = store.get(key);
  if (!hit) return undefined;
  if (hit.expires < Date.now()) {
    store.delete(key);
    return undefined;
  }
  return hit.value;
}
EOF
stash "add TTL support to cache"

# 2
cat > src/ui/Button.tsx <<'EOF'
export function Button({ label, onClick, variant = "primary" }: Props) {
  return (
    <button className={`btn btn--${variant}`} onClick={onClick}>
      {label}
    </button>
  );
}
EOF
stash "button variants, half done"

# 3
cat > src/api/client.ts <<'EOF'
export async function fetchUser(id: string, signal?: AbortSignal) {
  const res = await fetch(`/api/users/${id}`, { signal });
  if (!res.ok) throw new Error(`request failed: ${res.status}`);
  return res.json();
}
EOF
cat > src/api/retry.ts <<'EOF'
export async function withRetry<T>(fn: () => Promise<T>, attempts = 3) {
  let lastError: unknown;
  for (let i = 0; i < attempts; i++) {
    try {
      return await fn();
    } catch (err) {
      lastError = err;
    }
  }
  throw lastError;
}
EOF
stash "retry + abort for api client"

# 4
printf '\n.env.local\ncoverage/\n' >> .gitignore 2>/dev/null || printf '.env.local\ncoverage/\n' > .gitignore
$G add .gitignore 2>/dev/null || true
cat > src/ui/theme.css <<'EOF'
:root {
  --bg: #0d1117;
  --fg: #e6edf3;
  --accent: #2f81f7;
}
EOF
stash "dark theme spike"

# 5
cat > README.md <<'EOF'
# example-app

A small example. Setup:

    npm install
    npm run dev
EOF
stash "expand the readme"

$G stash list
