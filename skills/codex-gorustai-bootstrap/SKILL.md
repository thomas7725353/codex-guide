---
name: codex-gorustai-bootstrap
description: Use when a Windows or macOS user needs Codex Guide repair, GorustAI provider setup, Codex App or Codex CLI installation, cc-switch-cli installation, OPENAI_API_KEY environment repair, or remote operations support after running codex-guide.
---

# Codex GorustAI Bootstrap

You are repairing a non-technical user's Codex Guide setup. Be direct, use the user's OS, and avoid asking them to choose between technical options unless a command fails.

## Goal

Make this machine able to run:

- `codex`
- `codex app`
- `cc-switch --app codex`
- `codex-guide launch-codex`
- `codex-guide launch-cc-switch`

The required Codex provider config is:

```toml
model_provider = "gorustai"
model = "gpt-5.5"
review_model = "gpt-5.5"
model_reasoning_effort = "xhigh"
disable_response_storage = true
network_access = true
windows_wsl_setup_acknowledged = true

[model_providers.gorustai]
name = "OpenAI"
base_url = "https://gorustai.com"
wire_api = "responses"
requires_openai_auth = false
env_key = "OPENAI_API_KEY"

[features]
goals = true
```

Never invent or hard-code an API key. If the environment variable is missing, prompt the user to paste their own key into the installer prompt or current shell.

## First Pass

1. Identify OS and shell.
2. Run `codex-guide doctor` if available.
3. Run:
   - `codex --version`
   - `codex doctor`
   - `cc-switch --app codex env tools`
   - `cc-switch --app codex provider list`
4. Inspect:
   - Codex config: `${CODEX_HOME:-$HOME/.codex}/config.toml` on macOS, `%USERPROFILE%\.codex\config.toml` on Windows.
   - User skill folders: `$HOME/.agents/skills/codex-gorustai-bootstrap` and `${CODEX_HOME:-$HOME/.codex}/skills/codex-gorustai-bootstrap`.

## Repair Order

### 1. Install or repair codex-guide

Windows PowerShell:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://guide.gorustai.com/install.ps1 | iex"
```

macOS:

```bash
curl -fsSL https://guide.gorustai.com/install.sh | bash
```

If the user cannot use Terminal/PowerShell, tell them to download and double-click:

- Windows: `https://guide.gorustai.com/download/windows.zip`
- macOS Apple Silicon: `https://guide.gorustai.com/download/macos-arm64.dmg`
- macOS Intel: `https://guide.gorustai.com/download/macos-x64.dmg`

### 2. Install Codex App

Windows:

```powershell
winget install Codex -s msstore --accept-package-agreements --accept-source-agreements
```

If `winget` fails, open:

```text
https://get.microsoft.com/installer/download/9PLM9XGG6VKS?cid=website_cta_psi
```

macOS:

```bash
codex app
```

### 3. Install Codex CLI

Prefer the GorustAI guide mirror because some users cannot access `chatgpt.com` directly.

Windows:

```powershell
$ErrorActionPreference='Stop'
$env:CODEX_NON_INTERACTIVE='1'
$tmp=Join-Path $env:TEMP ('codex-install-' + [guid]::NewGuid().ToString() + '.ps1')
try {
  Invoke-WebRequest -Uri https://guide.gorustai.com/codex/install.ps1 -OutFile $tmp -ErrorAction Stop
} catch {
  Invoke-WebRequest -Uri https://github.com/openai/codex/releases/latest/download/install.ps1 -OutFile $tmp
}
& $tmp
```

macOS:

```bash
tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT
if curl -fsSL https://guide.gorustai.com/codex/install.sh -o "$tmp"; then
  :
else
  curl -fsSL https://github.com/openai/codex/releases/latest/download/install.sh -o "$tmp"
fi
CODEX_NON_INTERACTIVE=1 sh "$tmp"
```

After installation, refresh the current shell PATH:

Windows:

```powershell
$env:Path = "$env:LOCALAPPDATA\Programs\OpenAI\Codex\bin;$env:Path"
```

macOS:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### 4. Install cc-switch-cli

Windows:

```powershell
codex-guide setup
```

If only cc-switch is broken, rerun the full setup. It downloads `cc-switch-cli-windows-x64.zip` from `https://guide.gorustai.com/cc-switch/windows-x64.zip` and adds `%LOCALAPPDATA%\cc-switch\bin` to the user PATH.

macOS:

```bash
tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT
if curl -fsSL https://guide.gorustai.com/cc-switch/install.sh -o "$tmp"; then
  :
else
  curl -fsSL https://github.com/SaladDay/cc-switch-cli/releases/latest/download/install.sh -o "$tmp"
fi
bash "$tmp"
export PATH="$HOME/.local/bin:$PATH"
```

### 5. Repair environment and config

Windows PowerShell:

```powershell
[Environment]::SetEnvironmentVariable('OPENAI_API_KEY', $env:OPENAI_API_KEY, 'User')
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.codex" | Out-Null
@'
model_provider = "gorustai"
model = "gpt-5.5"
review_model = "gpt-5.5"
model_reasoning_effort = "xhigh"
disable_response_storage = true
network_access = true
windows_wsl_setup_acknowledged = true

[model_providers.gorustai]
name = "OpenAI"
base_url = "https://gorustai.com"
wire_api = "responses"
requires_openai_auth = false
env_key = "OPENAI_API_KEY"

[features]
goals = true
'@ | Set-Content -Encoding UTF8 "$env:USERPROFILE\.codex\config.toml"
```

macOS:

```bash
mkdir -p "${CODEX_HOME:-$HOME/.codex}"
cat > "${CODEX_HOME:-$HOME/.codex}/config.toml" <<'EOF'
model_provider = "gorustai"
model = "gpt-5.5"
review_model = "gpt-5.5"
model_reasoning_effort = "xhigh"
disable_response_storage = true
network_access = true
windows_wsl_setup_acknowledged = true

[model_providers.gorustai]
name = "OpenAI"
base_url = "https://gorustai.com"
wire_api = "responses"
requires_openai_auth = false
env_key = "OPENAI_API_KEY"

[features]
goals = true
EOF
```

If the user has not set the environment variable, prompt them for their own key and then set it:

Windows:

```powershell
[Environment]::SetEnvironmentVariable('OPENAI_API_KEY', '<USER_KEY>', 'User')
$env:OPENAI_API_KEY='<USER_KEY>'
```

macOS:

```bash
export OPENAI_API_KEY='<USER_KEY>'
```

Also persist it in `~/.zshrc`, `~/.bashrc`, or `~/.profile` using the user's current shell.

### 6. Final verification

Run:

```bash
codex --version
codex doctor
cc-switch --app codex provider import-live
cc-switch --app codex env tools
cc-switch --app codex provider list
```

A good final state has:

- `codex --version` prints a version.
- `codex doctor` runs.
- `cc-switch --app codex provider list` shows `default` or `gorustai` pointing at `https://gorustai.com`.
- The user can start remote support with `codex-guide launch-codex`.

## Failure Handling

- If `guide.gorustai.com` fails, retry official GitHub/OpenAI release URLs.
- If GitHub is blocked, ask the user to use the DMG/ZIP from `https://guide.gorustai.com`.
- If Windows PATH is stale, close and reopen PowerShell, or prepend the install directories in the current session.
- If macOS says a downloaded app cannot be opened, use right-click Open, or run the `.command` file from the mounted DMG.
- Do not delete user backups. `codex-guide` backs up old Codex configs with `.bak-<timestamp>`.
