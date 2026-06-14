#!/usr/bin/env bash
set -euo pipefail

repo="thomas7725353/codex-guide"
install_dir="${HOME}/.local/bin"
bin_path="${install_dir}/codex-guide"

echo "Codex Guide macOS 一键安装器"
echo "============================"

os="$(uname -s)"
arch="$(uname -m)"
if [[ "${os}" != "Darwin" ]]; then
  echo "当前脚本主要支持 macOS。其他 Unix 系统可以手动下载 release。" >&2
  exit 1
fi

case "${arch}" in
  arm64) asset="codex-guide-macos-arm64" ;;
  x86_64) asset="codex-guide-macos-x64" ;;
  *)
    echo "不支持的 macOS 架构: ${arch}" >&2
    exit 1
    ;;
esac

mkdir -p "${install_dir}"
tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

release_json="${tmp}/release.json"
curl -fsSL "https://api.github.com/repos/${repo}/releases/latest" -o "${release_json}"
download_url="$(python3 - "${release_json}" "${asset}" <<'PY'
import json, sys
data = json.load(open(sys.argv[1]))
asset_name = sys.argv[2]
for asset in data.get("assets", []):
    if asset.get("name") == asset_name:
        print(asset["browser_download_url"])
        break
else:
    raise SystemExit(f"missing asset: {asset_name}")
PY
)"

echo "下载 ${asset}"
curl -fL "${download_url}" -o "${bin_path}"
chmod +x "${bin_path}"

case ":${PATH}:" in
  *":${install_dir}:"*) ;;
  *)
    echo "提示：${install_dir} 还不在 PATH。请把下面这一行加入 ~/.zshrc："
    echo "export PATH=\"\$HOME/.local/bin:\$PATH\""
    export PATH="${install_dir}:${PATH}"
    ;;
esac

"${bin_path}" setup

