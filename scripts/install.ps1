$ErrorActionPreference = "Stop"

$InstallRoot = Join-Path $env:LOCALAPPDATA "codex-guide"
$BinDir = Join-Path $InstallRoot "bin"
$ExePath = Join-Path $BinDir "codex-guide.exe"
$MirrorExe = "https://guide.gorustai.com/download/windows.exe"
$GitHubApi = "https://api.github.com/repos/thomas7725353/codex-guide/releases/latest"

Write-Host "Codex Guide Windows 一键安装器"
Write-Host "================================"

if (-not [Environment]::Is64BitOperatingSystem) {
  throw "只支持 64 位 Windows。"
}

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

$Tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("codex-guide-" + [System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Force -Path $Tmp | Out-Null
$Download = Join-Path $Tmp "codex-guide.exe"

Write-Host "下载 codex-guide..."
try {
  Invoke-WebRequest -Uri $MirrorExe -OutFile $Download
} catch {
  Write-Host "镜像下载失败，改用 GitHub release..."
  $Release = Invoke-RestMethod -Uri $GitHubApi -Headers @{ "User-Agent" = "codex-guide-installer" }
  $Asset = $Release.assets | Where-Object { $_.name -eq "codex-guide-windows-x64.exe" } | Select-Object -First 1
  if (-not $Asset) {
    throw "没有找到 codex-guide-windows-x64.exe。请确认 GitHub Release 已发布。"
  }
  Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile $Download
}
Move-Item -Force $Download $ExePath

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (($UserPath -split ";") -notcontains $BinDir) {
  [Environment]::SetEnvironmentVariable("Path", (($UserPath.TrimEnd(";") + ";$BinDir").TrimStart(";")), "User")
  $env:Path = "$env:Path;$BinDir"
}

Write-Host "启动安装配置..."
& $ExePath setup

Write-Host ""
Write-Host "完成。故障远程运维时运行："
Write-Host "  codex-guide launch-codex"
Write-Host "也可以启动 cc-switch："
Write-Host "  codex-guide launch-cc-switch"
