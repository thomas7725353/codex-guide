use anyhow::{anyhow, bail, Context, Result};
use chrono::Local;
use clap::{Parser, Subcommand};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use zip::ZipArchive;

const REPO: &str = "thomas7725353/codex-guide";
const CC_SWITCH_REPO: &str = "SaladDay/cc-switch-cli";
const CODEX_WINDOWS_INSTALL_MIRROR: &str = "https://guide.gorustai.com/codex/install.ps1";
const CODEX_WINDOWS_INSTALL_OFFICIAL: &str =
    "https://github.com/openai/codex/releases/latest/download/install.ps1";
const CODEX_UNIX_INSTALL_MIRROR: &str = "https://guide.gorustai.com/codex/install.sh";
const CODEX_UNIX_INSTALL_OFFICIAL: &str =
    "https://github.com/openai/codex/releases/latest/download/install.sh";
const CODEX_APP_WINDOWS: &str =
    "https://get.microsoft.com/installer/download/9PLM9XGG6VKS?cid=website_cta_psi";
const CC_SWITCH_WINDOWS_MIRROR: &str = "https://guide.gorustai.com/cc-switch/windows-x64.zip";
const CC_SWITCH_UNIX_INSTALL_MIRROR: &str = "https://guide.gorustai.com/cc-switch/install.sh";
const CC_SWITCH_UNIX_INSTALL_OFFICIAL: &str =
    "https://github.com/SaladDay/cc-switch-cli/releases/latest/download/install.sh";
const DEFAULT_MODEL: &str = "gpt-5.5";
const DEFAULT_BASE_URL: &str = "https://gorustai.com";
const FALLBACK_SKILL: &str = include_str!("../skills/codex-gorustai-bootstrap/SKILL.md");

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the full beginner setup flow.
    Setup {
        /// Skip interactive prompts where possible.
        #[arg(long)]
        yes: bool,
        /// API key. Prefer interactive input instead of passing this on the command line.
        #[arg(long, env = "OPENAI_API_KEY")]
        api_key: Option<String>,
    },
    /// Print detected tool and config status.
    Doctor,
    /// Start Codex CLI for remote support or manual repair.
    LaunchCodex,
    /// Start cc-switch-cli in Codex mode.
    LaunchCcSwitch,
    /// Print the generated Codex config without writing it.
    PrintConfig,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Platform {
    Windows,
    Macos,
    Other,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Commands::Setup {
        yes: false,
        api_key: None,
    }) {
        Commands::Setup { yes, api_key } => setup(yes, api_key),
        Commands::Doctor => doctor(),
        Commands::LaunchCodex => launch_codex(),
        Commands::LaunchCcSwitch => launch_cc_switch(),
        Commands::PrintConfig => {
            print!("{}", codex_config());
            Ok(())
        }
    }
}

fn setup(yes: bool, api_key: Option<String>) -> Result<()> {
    print_header();
    let platform = platform();
    match platform {
        Platform::Windows => println!("检测到 Windows：按 Windows 优先流程安装。"),
        Platform::Macos => println!("检测到 macOS：按 macOS 流程安装。"),
        Platform::Other => println!("检测到其他系统：只做通用检查，推荐使用 Windows 或 macOS。"),
    }

    print_system_info();
    ensure_codex_app_before_cli(platform, yes)?;
    ensure_codex_cli(platform, yes)?;
    ensure_codex_app_after_cli(platform)?;
    ensure_cc_switch(platform, yes)?;
    let key = match api_key {
        Some(key) if !key.trim().is_empty() => key,
        _ => prompt_api_key()?,
    };
    set_openai_api_key(&key)?;
    write_codex_config()?;
    install_fallback_skill()?;

    println!();
    println!("正在做最后检查...");
    run_best_effort("codex", &["--version"]);
    run_best_effort("codex", &["doctor"]);
    run_best_effort("cc-switch", &["--app", "codex", "provider", "import-live"]);
    run_best_effort("cc-switch", &["--app", "codex", "env", "tools"]);
    run_best_effort("cc-switch", &["--app", "codex", "provider", "list"]);

    println!();
    println!("安装配置完成。");
    println!("下一步：重新打开 PowerShell/终端，然后运行：");
    println!("  codex-guide launch-codex");
    println!("  codex-guide launch-cc-switch");
    println!("  codex doctor");
    println!("  cc-switch --app codex");
    println!();
    println!("如果这台电脑环境有问题，运行下面这条，让 Codex CLI 帮你检查和修：");
    println!(
        "  codex \"$codex-gorustai-bootstrap 按 skill 检查并安装全套：Codex App、Codex CLI、cc-switch-cli、OPENAI_API_KEY 环境变量和 gorustai provider 配置。\""
    );
    if platform == Platform::Windows {
        println!();
        println!("Windows 桌面 App 下载地址：{CODEX_APP_WINDOWS}");
        println!("也可以运行：winget install Codex -s msstore");
    }
    Ok(())
}

fn doctor() -> Result<()> {
    print_header();
    print_system_info();
    println!("codex: {}", status_for_command("codex"));
    println!("cc-switch: {}", status_for_command("cc-switch"));
    println!("winget: {}", status_for_command("winget"));
    println!("git: {}", status_for_command("git"));
    println!("node: {}", status_for_command("node"));
    println!("Codex config: {}", codex_config_path().display());
    println!("Codex auth: {}", codex_auth_path().display());
    Ok(())
}

fn launch_codex() -> Result<()> {
    if !command_exists("codex") {
        bail!("没有找到 codex。请先运行 codex-guide setup 安装 Codex CLI。");
    }
    println!("启动 Codex CLI。远程运维时，把这个窗口里的报错和提示发给技术人员。");
    run_command_interactive("codex", &[])
}

fn launch_cc_switch() -> Result<()> {
    if !command_exists("cc-switch") {
        bail!("没有找到 cc-switch。请先运行 codex-guide setup 安装 cc-switch-cli。");
    }
    println!("启动 cc-switch-cli Codex 管理界面。");
    run_command_interactive("cc-switch", &["--app", "codex"])
}

fn print_header() {
    println!("Codex Guide 一键安装器");
    println!("======================");
    println!("本工具不会内置 API key。请只在自己电脑上输入 key。");
    println!();
}

fn platform() -> Platform {
    if cfg!(windows) {
        Platform::Windows
    } else if cfg!(target_os = "macos") {
        Platform::Macos
    } else {
        Platform::Other
    }
}

fn print_system_info() {
    println!("系统: {} {}", env::consts::OS, env::consts::ARCH);
    if let Some(home) = dirs::home_dir() {
        println!("用户目录: {}", home.display());
    }
    println!("codex: {}", status_for_command("codex"));
    println!("cc-switch: {}", status_for_command("cc-switch"));
    if cfg!(windows) {
        println!("winget: {}", status_for_command("winget"));
    }
    println!();
}

fn status_for_command(cmd: &str) -> &'static str {
    if command_exists(cmd) {
        "已安装"
    } else {
        "未找到"
    }
}

fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success() || !status.success())
        .unwrap_or(false)
}

fn refresh_codex_cli_path(platform: Platform) {
    if let Some(dir) = env::var_os("CODEX_INSTALL_DIR").filter(|value| !value.is_empty()) {
        prepend_to_process_path(&PathBuf::from(dir));
    }

    match platform {
        Platform::Windows => {
            if let Ok(local) = local_appdata() {
                prepend_to_process_path(
                    &local
                        .join("Programs")
                        .join("OpenAI")
                        .join("Codex")
                        .join("bin"),
                );
            }
        }
        Platform::Macos | Platform::Other => {
            if let Some(home) = dirs::home_dir() {
                prepend_to_process_path(&home.join(".local").join("bin"));
            }
        }
    }
}

fn refresh_cc_switch_path(platform: Platform) {
    match platform {
        Platform::Windows => {
            if let Ok(local) = local_appdata() {
                prepend_to_process_path(&local.join("cc-switch").join("bin"));
            }
        }
        Platform::Macos | Platform::Other => {
            if let Some(home) = dirs::home_dir() {
                prepend_to_process_path(&home.join(".local").join("bin"));
            }
        }
    }
}

fn prepend_to_process_path(dir: &Path) {
    let current = env::var_os("PATH").unwrap_or_default();
    let current_paths: Vec<PathBuf> = env::split_paths(&current).collect();
    if current_paths.iter().any(|path| path == dir) {
        return;
    }

    let mut paths = Vec::with_capacity(current_paths.len() + 1);
    paths.push(dir.to_path_buf());
    paths.extend(current_paths);
    if let Ok(joined) = env::join_paths(paths) {
        env::set_var("PATH", joined);
    }
}

fn ensure_codex_app_before_cli(platform: Platform, yes: bool) -> Result<()> {
    match platform {
        Platform::Windows => ensure_codex_app_windows(yes),
        Platform::Macos | Platform::Other => Ok(()),
    }
}

fn ensure_codex_app_after_cli(platform: Platform) -> Result<()> {
    match platform {
        Platform::Windows => {
            if codex_app_installed_windows() {
                return Ok(());
            }
            println!("尝试用 Codex CLI 继续安装/打开 Windows 桌面 App...");
            run_best_effort("codex", &["app", "--download-url", CODEX_APP_WINDOWS]);
            Ok(())
        }
        Platform::Macos => ensure_codex_app_macos(),
        Platform::Other => Ok(()),
    }
}

fn ensure_codex_app_windows(yes: bool) -> Result<()> {
    println!("检查 Codex Windows 桌面 App...");
    if !command_exists("winget") {
        println!("未找到 winget，无法自动安装 Codex App。");
        println!("请手动打开下载链接：{CODEX_APP_WINDOWS}");
        return Ok(());
    }

    if codex_app_installed_windows() {
        println!("Codex Windows App 看起来已安装。");
        return Ok(());
    }

    println!("没有检测到 Codex Windows App。下载链接：{CODEX_APP_WINDOWS}");
    if !yes {
        println!("现在自动安装 Codex App。这个过程可能会弹出 Microsoft Store 提示。");
    }

    let result = run_command(
        "winget",
        &[
            "install",
            "Codex",
            "-s",
            "msstore",
            "--accept-package-agreements",
            "--accept-source-agreements",
        ],
    );
    if let Err(error) = result {
        println!("Codex App 自动安装失败：{error}");
        println!("请手动打开下载链接：{CODEX_APP_WINDOWS}");
    }
    Ok(())
}

fn codex_app_installed_windows() -> bool {
    let script = "if ((Get-AppxPackage -ErrorAction SilentlyContinue | Where-Object { $_.Name -like '*Codex*' -or $_.PackageFullName -like '*Codex*' }) -or (Get-StartApps -ErrorAction SilentlyContinue | Where-Object { $_.Name -like '*Codex*' })) { exit 0 } else { exit 1 }";
    let appx_ok = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    if appx_ok {
        return true;
    }

    if !command_exists("winget") {
        return false;
    }

    let output = Command::new("winget")
        .args([
            "list",
            "Codex",
            "-s",
            "msstore",
            "--accept-source-agreements",
        ])
        .output();
    let Ok(output) = output else {
        return false;
    };
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
    .to_lowercase();
    output.status.success()
        && text.contains("codex")
        && !text.contains("no installed package")
        && !text.contains("no package found")
}

fn ensure_codex_app_macos() -> Result<()> {
    println!("检查 Codex macOS 桌面 App...");
    if command_exists("codex") {
        run_best_effort("codex", &["app"]);
    } else {
        println!("Codex CLI 安装后可运行 `codex app` 安装/打开桌面 App。");
    }
    Ok(())
}

fn ensure_codex_cli(platform: Platform, yes: bool) -> Result<()> {
    if command_exists("codex") {
        println!("Codex CLI 已安装，跳过。");
        return Ok(());
    }

    println!("Codex CLI 未找到，准备安装。");
    let _ = yes;

    match platform {
        Platform::Windows => {
            let script = format!(
                "$ErrorActionPreference='Stop'; $env:CODEX_NON_INTERACTIVE='1'; $tmp=Join-Path $env:TEMP ('codex-install-' + [guid]::NewGuid().ToString() + '.ps1'); try {{ Invoke-WebRequest -Uri {CODEX_WINDOWS_INSTALL_MIRROR} -OutFile $tmp -ErrorAction Stop }} catch {{ Write-Host '镜像下载失败，改用官方安装脚本...'; Invoke-WebRequest -Uri {CODEX_WINDOWS_INSTALL_OFFICIAL} -OutFile $tmp }}; & $tmp"
            );
            run_command(
                "powershell",
                &[
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    &script,
                ],
            )
            .context("Codex CLI 安装失败。请尝试用管理员 PowerShell 运行 README 里的手动命令。")?;
        }
        Platform::Macos | Platform::Other => {
            let script = format!(
                "tmp=\"$(mktemp)\"; trap 'rm -f \"$tmp\"' EXIT; if curl -fsSL {CODEX_UNIX_INSTALL_MIRROR} -o \"$tmp\"; then :; else curl -fsSL {CODEX_UNIX_INSTALL_OFFICIAL} -o \"$tmp\"; fi; CODEX_NON_INTERACTIVE=1 sh \"$tmp\""
            );
            run_command("sh", &["-c", &script])
                .context("Codex CLI 安装失败。请检查网络或手动运行官方安装命令。")?;
        }
    }
    refresh_codex_cli_path(platform);
    if !command_exists("codex") {
        bail!("Codex CLI 安装后仍未找到。请确认网络可访问 guide.gorustai.com，或使用 macOS DMG/Windows zip 离线包重新安装。");
    }
    Ok(())
}

fn ensure_cc_switch(platform: Platform, yes: bool) -> Result<()> {
    if command_exists("cc-switch") {
        println!("cc-switch 已安装，跳过。");
        return Ok(());
    }

    println!("cc-switch 未找到，准备安装。");
    let _ = yes;

    match platform {
        Platform::Windows => install_cc_switch_windows(),
        Platform::Macos | Platform::Other => install_cc_switch_unix(),
    }?;
    refresh_cc_switch_path(platform);
    if !command_exists("cc-switch") {
        bail!("cc-switch 安装后仍未找到。请重新打开终端，或手动运行 cc-switch 安装命令。");
    }
    Ok(())
}

fn install_cc_switch_windows() -> Result<()> {
    println!("下载 cc-switch Windows x64。");
    let root = local_appdata()?.join("cc-switch");
    let bin = root.join("bin");
    fs::create_dir_all(&bin).with_context(|| format!("创建目录失败: {}", bin.display()))?;

    let tmp = tempfile::tempdir()?;
    let zip_path = tmp.path().join("cc-switch-cli-windows-x64.zip");
    if let Err(error) = download_to(CC_SWITCH_WINDOWS_MIRROR, &zip_path) {
        println!("cc-switch 镜像下载失败：{error}");
        let release = latest_release(CC_SWITCH_REPO)?;
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == "cc-switch-cli-windows-x64.zip")
            .or_else(|| {
                release
                    .assets
                    .iter()
                    .find(|asset| asset.name.ends_with("windows-x64.zip"))
            })
            .ok_or_else(|| anyhow!("没有找到 cc-switch Windows x64 release 资产"))?;
        println!(
            "改用 GitHub release 下载 {}: {}",
            release.tag_name, asset.name
        );
        download_to(&asset.browser_download_url, &zip_path)?;
    }
    unzip_single_binary(&zip_path, "cc-switch.exe", &bin.join("cc-switch.exe"))?;
    add_to_user_path(&bin)?;
    println!("cc-switch 已安装到 {}", bin.display());
    Ok(())
}

fn install_cc_switch_unix() -> Result<()> {
    let script = format!(
        "tmp=\"$(mktemp)\"; trap 'rm -f \"$tmp\"' EXIT; if curl -fsSL {CC_SWITCH_UNIX_INSTALL_MIRROR} -o \"$tmp\"; then :; else curl -fsSL {CC_SWITCH_UNIX_INSTALL_OFFICIAL} -o \"$tmp\"; fi; bash \"$tmp\""
    );
    run_command("sh", &["-c", &script])
        .context("cc-switch 安装失败。请检查网络或手动下载 release。")
}

fn latest_release(repo: &str) -> Result<GithubRelease> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .user_agent(format!("codex-guide/{}", env!("CARGO_PKG_VERSION")))
        .build()?;
    let response = client.get(url).send()?.error_for_status()?;
    Ok(response.json()?)
}

fn download_to(url: &str, path: &Path) -> Result<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(180))
        .user_agent(format!("codex-guide/{}", env!("CARGO_PKG_VERSION")))
        .build()?;
    let mut response = client.get(url).send()?.error_for_status()?;
    let mut file =
        fs::File::create(path).with_context(|| format!("创建下载文件失败: {}", path.display()))?;
    io::copy(&mut response, &mut file).context("下载写入失败")?;
    Ok(())
}

fn unzip_single_binary(zip_path: &Path, binary_name: &str, dest: &Path) -> Result<()> {
    let file = fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let Some(name) = entry.enclosed_name() else {
            continue;
        };
        if name.file_name().and_then(|v| v.to_str()) == Some(binary_name) {
            let mut out = fs::File::create(dest)?;
            io::copy(&mut entry, &mut out)?;
            return Ok(());
        }
    }
    bail!("zip 里没有找到 {binary_name}");
}

fn write_codex_config() -> Result<()> {
    let dir = codex_home();
    fs::create_dir_all(&dir).with_context(|| format!("创建 Codex 目录失败: {}", dir.display()))?;
    let path = codex_config_path();
    if path.exists() {
        let backup =
            path.with_extension(format!("toml.bak-{}", Local::now().format("%Y%m%d-%H%M%S")));
        fs::copy(&path, &backup).with_context(|| format!("备份失败: {}", backup.display()))?;
        println!("已备份旧配置: {}", backup.display());
    }
    fs::write(&path, codex_config())
        .with_context(|| format!("写入配置失败: {}", path.display()))?;
    println!("已写入 Codex 配置: {}", path.display());
    Ok(())
}

fn install_fallback_skill() -> Result<()> {
    if FALLBACK_SKILL.trim().is_empty() {
        println!("当前二进制未嵌入兜底 skill，跳过 skill 安装。");
        return Ok(());
    }

    let text = FALLBACK_SKILL;
    let home = dirs::home_dir().ok_or_else(|| anyhow!("无法获取用户目录"))?;
    let mut skill_dirs = vec![home
        .join(".agents")
        .join("skills")
        .join("codex-gorustai-bootstrap")];

    let codex_home_skill_dir = codex_home().join("skills").join("codex-gorustai-bootstrap");
    if !skill_dirs.iter().any(|dir| dir == &codex_home_skill_dir) {
        skill_dirs.push(codex_home_skill_dir);
    }

    for skill_dir in skill_dirs {
        fs::create_dir_all(&skill_dir)
            .with_context(|| format!("创建 skill 目录失败: {}", skill_dir.display()))?;
        let path = skill_dir.join("SKILL.md");
        fs::write(&path, text)
            .with_context(|| format!("写入兜底 skill 失败: {}", path.display()))?;
        println!("已安装 Codex 兜底 skill: {}", path.display());
    }
    Ok(())
}

fn set_openai_api_key(api_key: &str) -> Result<()> {
    let key = api_key.trim();
    if key.is_empty() {
        bail!("API key 不能为空");
    }
    if cfg!(windows) {
        let script = format!(
            "[Environment]::SetEnvironmentVariable('OPENAI_API_KEY', '{}', 'User')",
            escape_powershell_single_quoted(key)
        );
        run_command(
            "powershell",
            &[
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &script,
            ],
        )
        .context("写入用户 OPENAI_API_KEY 环境变量失败")?;
        env::set_var("OPENAI_API_KEY", key);
        println!("已写入用户级 OPENAI_API_KEY。重新打开终端后永久生效。");
    } else {
        env::set_var("OPENAI_API_KEY", key);
        persist_openai_api_key_unix(key)?;
        println!("已写入 shell 配置里的 OPENAI_API_KEY。重新打开终端后永久生效。");
    }
    Ok(())
}

fn codex_config() -> String {
    format!(
        r#"model_provider = "gorustai"
model = "{DEFAULT_MODEL}"
review_model = "{DEFAULT_MODEL}"
model_reasoning_effort = "xhigh"
disable_response_storage = true
network_access = true
windows_wsl_setup_acknowledged = true

[model_providers.gorustai]
name = "OpenAI"
base_url = "{DEFAULT_BASE_URL}"
wire_api = "responses"
requires_openai_auth = false
env_key = "OPENAI_API_KEY"

[features]
goals = true
"#
    )
}

fn prompt_api_key() -> Result<String> {
    println!();
    println!("请输入你的 OPENAI_API_KEY。输入时不会显示在屏幕上。");
    let key = rpassword::prompt_password("OPENAI_API_KEY: ")?;
    if !key.trim().is_empty() {
        return Ok(key);
    }
    print!("没有读到 key，请重新粘贴 OPENAI_API_KEY: ");
    io::stdout().flush()?;
    let mut fallback = String::new();
    io::stdin().read_line(&mut fallback)?;
    Ok(fallback)
}

fn codex_home() -> PathBuf {
    env::var_os("CODEX_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().expect("无法获取用户目录").join(".codex"))
}

fn codex_config_path() -> PathBuf {
    codex_home().join("config.toml")
}

fn codex_auth_path() -> PathBuf {
    codex_home().join("auth.json")
}

fn escape_powershell_single_quoted(value: &str) -> String {
    value.replace('\'', "''")
}

fn persist_openai_api_key_unix(key: &str) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("无法获取用户目录"))?;
    let shell = env::var("SHELL").unwrap_or_default();
    let profile = if shell.contains("zsh") {
        home.join(".zshrc")
    } else if shell.contains("bash") {
        home.join(".bashrc")
    } else {
        home.join(".profile")
    };
    let marker_start = "# >>> codex-guide OPENAI_API_KEY >>>";
    let marker_end = "# <<< codex-guide OPENAI_API_KEY <<<";
    let export_line = format!(
        "{marker_start}\nexport OPENAI_API_KEY='{}'\n{marker_end}\n",
        key.replace('\'', "'\"'\"'")
    );
    let existing = fs::read_to_string(&profile).unwrap_or_default();
    let cleaned = remove_marked_block(&existing, marker_start, marker_end);
    fs::write(&profile, format!("{}\n{}", cleaned.trim_end(), export_line))
        .with_context(|| format!("写入 shell 配置失败: {}", profile.display()))?;
    Ok(())
}

fn remove_marked_block(text: &str, start: &str, end: &str) -> String {
    let mut out = Vec::new();
    let mut skipping = false;
    for line in text.lines() {
        if line.trim() == start {
            skipping = true;
            continue;
        }
        if line.trim() == end {
            skipping = false;
            continue;
        }
        if !skipping {
            out.push(line);
        }
    }
    out.join("\n")
}

fn local_appdata() -> Result<PathBuf> {
    if let Some(value) = env::var_os("LOCALAPPDATA") {
        return Ok(PathBuf::from(value));
    }
    dirs::data_local_dir().ok_or_else(|| anyhow!("无法定位 LOCALAPPDATA"))
}

fn add_to_user_path(dir: &Path) -> Result<()> {
    if !cfg!(windows) {
        return Ok(());
    }
    let dir_str = dir.to_string_lossy();
    let script = format!(
        "$p=[Environment]::GetEnvironmentVariable('Path','User'); if (($p -split ';') -notcontains '{dir_str}') {{ [Environment]::SetEnvironmentVariable('Path', (($p.TrimEnd(';') + ';{dir_str}').TrimStart(';')), 'User') }}"
    );
    run_command(
        "powershell",
        &[
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ],
    )
    .context("写入用户 PATH 失败")?;
    Ok(())
}

fn run_command(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd).args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        bail!("命令失败: {cmd} {}", args.join(" "))
    }
}

fn run_command_interactive(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd).args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        bail!("命令退出: {cmd} {}", args.join(" "))
    }
}

fn run_best_effort(cmd: &str, args: &[&str]) {
    println!("> {} {}", cmd, args.join(" "));
    match Command::new(cmd).args(args).status() {
        Ok(status) if status.success() => println!("  OK"),
        Ok(status) => println!("  未通过，退出码: {status}"),
        Err(error) => println!("  无法运行: {error}"),
    }
}

#[allow(dead_code)]
fn repo_name() -> &'static str {
    REPO
}
