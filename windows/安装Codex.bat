@echo off
chcp 65001 >nul
setlocal

cd /d "%~dp0"

echo Codex Guide Windows 双击安装器
echo ==============================
echo.

if not exist "codex-guide.exe" (
  echo 没找到 codex-guide.exe。
  echo 请确认你解压的是完整的 codex-guide-windows-x64.zip。
  pause
  exit /b 1
)

if exist "codex-gorustai-bootstrap\SKILL.md" (
  mkdir "%USERPROFILE%\.agents\skills\codex-gorustai-bootstrap" >nul 2>nul
  mkdir "%USERPROFILE%\.codex\skills\codex-gorustai-bootstrap" >nul 2>nul
  copy /y "codex-gorustai-bootstrap\SKILL.md" "%USERPROFILE%\.agents\skills\codex-gorustai-bootstrap\SKILL.md" >nul
  copy /y "codex-gorustai-bootstrap\SKILL.md" "%USERPROFILE%\.codex\skills\codex-gorustai-bootstrap\SKILL.md" >nul
  echo 已安装 Codex 兜底 skill。
  echo.
)

codex-guide.exe setup

echo.
echo 安装流程结束。
echo.
echo 如果电脑出故障或需要远程运维，请选择 1，直接启动 Codex CLI。
echo.
echo 请选择下一步：
echo   1. 启动 Codex CLI
echo   2. 启动 cc-switch-cli
echo   3. 退出
set /p choice=请输入 1/2/3 后回车：
if "%choice%"=="1" (
  codex-guide.exe launch-codex
  pause
  exit /b 0
)
if "%choice%"=="2" (
  codex-guide.exe launch-cc-switch
  pause
  exit /b 0
)

echo 以后可以重新打开 PowerShell，然后运行 codex-guide launch-codex 或 codex-guide launch-cc-switch。
pause
