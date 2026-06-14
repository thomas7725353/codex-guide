# Codex Guide 一键安装器

给 Windows 小白用户优先设计的一键安装脚本：安装 Codex App、安装 Codex CLI、安装 `cc-switch-cli`、写入 `gorustai.com` Codex 配置，并让用户在本机输入自己的 API key。

仓库不会保存真实 API key。

## 中国用户入口

如果 GitHub 打不开，直接访问：

https://guide.gorustai.com

这个 Cloudflare Worker 页面包含 Windows 下载、macOS 安装命令、Codex App 下载和 `cc-switch-cli` 镜像入口。

## Windows 最简单：下载后双击

大部分用户用这个方式，不需要会 PowerShell。

1. 下载 Windows 安装包：

   https://guide.gorustai.com/download/windows.zip

   GitHub 备用地址：

   https://github.com/thomas7725353/codex-guide/releases/latest/download/codex-guide-windows-x64.zip

2. 解压 zip。
3. 双击里面的 `安装Codex.bat`。
4. 按提示粘贴你的 `OPENAI_API_KEY`。

安装包会先检查有没有 Codex App，没有就自动安装；然后安装 Codex CLI、`cc-switch-cli`，并写好 Codex 配置。

## Windows 一行安装

打开 PowerShell，粘贴：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://guide.gorustai.com/install.ps1 | iex"
```

安装器会自动：

- 下载本项目最新 `codex-guide.exe`
- 检查 Windows、`winget`、Codex App、`codex`、`cc-switch`
- 自动安装 Codex Windows 桌面 App
- 安装 Codex CLI
- 下载并安装 `cc-switch-cli`
- 写入 `%USERPROFILE%\.codex\config.toml`
- 让你输入 `OPENAI_API_KEY`
- 运行 `codex --version`、`codex doctor` 和 `cc-switch --app codex env tools`

完成后重新打开 PowerShell，运行：

```powershell
codex-guide launch-codex
codex-guide launch-cc-switch
codex doctor
cc-switch --app codex
```

故障远程运维时，让用户运行或双击菜单选择：

```powershell
codex-guide launch-codex
```

如果电脑环境有问题，运行：

```text
$codex-gorustai-bootstrap 按 skill 检查并安装全套：Codex App、Codex CLI、cc-switch-cli、OPENAI_API_KEY 环境变量和 gorustai provider 配置。
```

安装器会把兜底 skill 安装到用户本机的 `~/.agents/skills/codex-gorustai-bootstrap/SKILL.md`。源码仓库不保存 skill 明文。

## Codex 下载地址

Windows 用户如果自动安装失败，直接打开这个链接：

https://get.microsoft.com/installer/download/9PLM9XGG6VKS?cid=website_cta_psi

也可以在 PowerShell 运行：

```powershell
winget install Codex -s msstore
```

Codex CLI 官方安装命令：

```powershell
$env:CODEX_NON_INTERACTIVE=1; irm https://chatgpt.com/codex/install.ps1 | iex
```

macOS / Linux:

```bash
curl -fsSL https://chatgpt.com/codex/install.sh | CODEX_NON_INTERACTIVE=1 sh
```

## macOS 一行安装

打开终端，粘贴：

```bash
curl -fsSL https://guide.gorustai.com/install.sh | bash
```

macOS 安装完成后同样可以运行：

```bash
codex-guide launch-codex
codex-guide launch-cc-switch
```

## Windows 常见问题

如果提示脚本不能运行，用管理员 PowerShell 执行：

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned
```

如果 `codex` 或 `cc-switch` 显示找不到，先关闭 PowerShell 再重新打开。

如果 `winget` 找不到，请更新 Windows，或安装 Microsoft 的 App Installer / Windows Package Manager。

如果杀毒软件拦截下载的 exe/zip，请到 Windows 安全中心查看拦截记录，确认来源是本仓库和 `SaladDay/cc-switch-cli` 后再放行。

Codex Windows 桌面 App 下载地址：

https://get.microsoft.com/installer/download/9PLM9XGG6VKS?cid=website_cta_psi

也可以运行：

```powershell
winget install Codex -s msstore
```

## 配置内容

安装器会写入：

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

旧配置会自动备份成 `.bak-时间戳`。

## 手动下载地址

- Codex CLI 官方安装脚本：
  - Windows: `https://chatgpt.com/codex/install.ps1`
  - macOS/Linux: `https://chatgpt.com/codex/install.sh`
- Codex Windows App: https://get.microsoft.com/installer/download/9PLM9XGG6VKS?cid=website_cta_psi
- `cc-switch-cli` releases: https://github.com/SaladDay/cc-switch-cli/releases

## Cloudflare Worker 部署

教程和镜像入口部署在：

https://guide.gorustai.com

Worker 源码在 `worker/src/index.js`，域名绑定在 `wrangler.toml`。改教程、脚本、Worker 或 README 后，GitHub Actions 会自动运行 `Deploy Worker` 并部署到 Cloudflare。

需要的 GitHub secrets：

- `CLOUDFLARE_ACCOUNT_ID`
- `CLOUDFLARE_API_TOKEN`
- `CODEX_GUIDE_FALLBACK_SKILL_B64`

Cloudflare token 不要写进仓库。
