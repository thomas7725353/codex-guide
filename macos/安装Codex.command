#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

echo "Codex Guide macOS 双击安装器"
echo "============================"
echo

if [[ ! -x "./codex-guide" ]]; then
  echo "没找到 codex-guide。请确认你打开的是完整的 Codex Guide 安装包。"
  read -r -p "按回车退出..."
  exit 1
fi

install_dir="${HOME}/.local/bin"
mkdir -p "${install_dir}"
cp -f "./codex-guide" "${install_dir}/codex-guide"
chmod +x "${install_dir}/codex-guide"
export PATH="${install_dir}:${PATH}"

"${install_dir}/codex-guide" setup

echo
echo "安装流程结束。"
echo
echo "请选择下一步："
echo "  1. 启动 Codex CLI"
echo "  2. 启动 cc-switch-cli"
echo "  3. 退出"
read -r -p "请输入 1/2/3 后回车：" choice
case "${choice}" in
  1) "${install_dir}/codex-guide" launch-codex ;;
  2) "${install_dir}/codex-guide" launch-cc-switch ;;
  *) echo "以后可以在终端运行 codex-guide launch-codex 或 codex-guide launch-cc-switch。" ;;
esac

echo
read -r -p "按回车退出..."
