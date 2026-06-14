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

codex-guide.exe setup

echo.
echo 安装流程结束。如果上面没有红色错误，请重新打开 PowerShell，然后运行 codex。
pause

