# Windows-first Codex Guide Design

## Goal

Build a beginner-friendly installer and guide for setting up Codex App, Codex CLI, the `gorustai.com` OpenAI-compatible endpoint, and `cc-switch-cli`.

Most users are expected to be on Windows, so Windows PowerShell is the primary path. macOS is supported as a secondary path.

## Secret handling

The repository, release artifacts, install scripts, and documentation must not contain a real API key. The installer asks the customer to paste their key locally at runtime.

The installer writes the key only on the customer's machine. It should avoid printing the key back to the terminal.

## Deliverables

- `codex-guide` Rust CLI.
- `scripts/install.ps1` as the primary Windows one-line installer, intended to run with `irm ... | iex`.
- `scripts/install.sh` as the secondary macOS one-line installer, intended to run with `curl -fsSL ... | bash`.
- GitHub Actions workflows for checking and building Windows/macOS release binaries.
- Chinese README aimed at non-technical users.
- Binary-embedded fallback skill installed to the user's `$HOME/.agents/skills/codex-gorustai-bootstrap/SKILL.md`, injected at release build time from a GitHub Actions secret so the source repository does not expose the skill text.

## Windows flow

The Windows path is optimized for a user who can open PowerShell and paste one command.

The public one-line command is:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/thomas7725353/codex-guide/main/scripts/install.ps1 | iex"
```

The script checks the system, downloads the latest `codex-guide.exe` release asset to `%LOCALAPPDATA%\codex-guide\bin`, adds that directory to the user `PATH`, then launches:

```powershell
codex-guide.exe setup
```

The Rust CLI then:

1. Detects Windows version, architecture, PowerShell, `winget`, Codex App, `codex`, and `cc-switch`.
2. Installs the Codex app through `winget install Codex -s msstore` when available, and always prints the direct Windows download link as fallback.
3. Installs Codex CLI using the official PowerShell path when possible, or gives exact fallback instructions.
4. Installs `cc-switch.exe` by downloading `cc-switch-cli-windows-x64.zip` from the latest GitHub release.
5. Adds `%LOCALAPPDATA%\codex-guide\bin` and `%LOCALAPPDATA%\cc-switch\bin` to the user `PATH`.
6. Prompts for `OPENAI_API_KEY`.
7. Backs up `%USERPROFILE%\.codex\config.toml` when it exists.
8. Writes a Windows-native Codex config using the selected provider, model, review model, reasoning effort, and `gorustai.com` base URL.
9. Runs diagnostics: `codex --version`, `codex doctor`, `cc-switch --app codex env tools`, `cc-switch --app codex provider import-live`, and prints next commands.
10. Installs the embedded `$codex-gorustai-bootstrap` fallback skill to the user's home directory so Codex CLI can repair a broken setup later.

The Windows path should not force WSL. WSL appears only as an advanced fallback for users who already need Linux-native workflows.

## macOS flow

The public one-line command is:

```bash
curl -fsSL https://raw.githubusercontent.com/thomas7725353/codex-guide/main/scripts/install.sh | bash
```

The macOS installer checks the system, downloads the latest `codex-guide` release asset into `~/.local/bin`, ensures the directory is on PATH where possible, and runs `codex-guide setup`.

The Rust CLI can install `cc-switch` through its upstream `install.sh`, and install or validate Codex CLI using the official shell installer.

## Codex configuration

The generated `config.toml` uses:

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

The installer also stores the API key in the local Codex auth path only after explicit local user input. If the local Codex version requires `codex login --api-key`, the installer should call it or show the exact command.

## Error handling

Errors are written in direct Chinese with the next action first.

Common Windows cases:

- PowerShell execution policy blocks scripts.
- `winget` is missing.
- `codex` or `cc-switch` is not on PATH until a new terminal opens.
- Git or Node.js is missing.
- Antivirus blocks downloaded `.exe` or `.zip`.
- User ran PowerShell without administrator rights when installing the Codex app or developer tools.
- Network cannot reach GitHub or Microsoft Store.

## Git and release policy

Do not build release artifacts on the local machine for delivery. GitHub Actions builds release binaries for:

- `x86_64-pc-windows-msvc`
- `aarch64-apple-darwin`
- `x86_64-apple-darwin`

The remote repository is `thomas7725353/codex-guide`. If it does not exist, create it before pushing.

`docs/superpowers/plans/` stays local-only and ignored.

## Acceptance checks

- `cargo fmt --check` passes.
- `cargo test` passes.
- `cargo clippy -- -D warnings` passes when dependencies are available.
- `scripts/install.ps1` does not contain secrets and supports `irm ... | iex`.
- `scripts/install.sh` does not contain secrets and supports `curl ... | bash`.
- README contains Windows-first steps and macOS secondary steps.
- No real API key appears in tracked files.
