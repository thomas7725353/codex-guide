#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

version="$(tr -d '[:space:]' < codex-version.txt)"
target="$(tr -d '[:space:]' < codex-target.txt)"
archive="codex-package-${target}.tar.gz"
manifest="codex-package_SHA256SUMS"

if [[ -z "${version}" || -z "${target}" ]]; then
  echo "Codex CLI offline package metadata is missing." >&2
  exit 1
fi

if [[ ! -f "${archive}" || ! -f "${manifest}" ]]; then
  echo "Codex CLI offline package is missing from this DMG." >&2
  exit 1
fi

expected="$(awk -v asset="${archive}" '$2 == asset { print tolower($1); found = 1; exit } END { if (!found) exit 1 }' "${manifest}")"
actual="$(shasum -a 256 "${archive}" | awk '{ print tolower($1) }')"
if [[ "${actual}" != "${expected}" ]]; then
  echo "Codex CLI offline package checksum mismatch." >&2
  echo "expected: ${expected}" >&2
  echo "actual:   ${actual}" >&2
  exit 1
fi

codex_home="${CODEX_HOME:-${HOME}/.codex}"
standalone_root="${codex_home}/packages/standalone"
releases_dir="${standalone_root}/releases"
release_name="${version}-${target}"
release_dir="${releases_dir}/${release_name}"
current_link="${standalone_root}/current"
bin_dir="${CODEX_INSTALL_DIR:-${HOME}/.local/bin}"
bin_path="${bin_dir}/codex"

mkdir -p "${releases_dir}" "${bin_dir}"
stage="$(mktemp -d "${releases_dir}/.staging.${release_name}.XXXXXX")"
cleanup() {
  rm -rf "${stage}" 2>/dev/null || true
}
trap cleanup EXIT

tar -xzf "${archive}" -C "${stage}"
chmod 0755 "${stage}/bin/codex" "${stage}/codex-path/rg"
ln -sf "bin/codex" "${stage}/codex"

rm -rf "${release_dir}"
mv "${stage}" "${release_dir}"
trap - EXIT

tmp_link="${standalone_root}/.current.$$"
rm -f "${tmp_link}"
ln -s "${release_dir}" "${tmp_link}"
rm -rf "${current_link}"
mv -f "${tmp_link}" "${current_link}"

rm -f "${bin_path}"
ln -s "${current_link}/bin/codex" "${bin_path}"
export PATH="${bin_dir}:${PATH}"

profile="${HOME}/.profile"
case "${SHELL:-}" in
  */zsh) profile="${HOME}/.zprofile" ;;
  */bash) profile="${HOME}/.bash_profile" ;;
esac

begin_marker="# >>> Codex installer >>>"
end_marker="# <<< Codex installer <<<"
path_line='export PATH="$HOME/.local/bin:$PATH"'
if [[ ! -f "${profile}" ]] || ! grep -F "${begin_marker}" "${profile}" >/dev/null 2>&1; then
  {
    printf '\n%s\n' "${begin_marker}"
    printf '%s\n' "${path_line}"
    printf '%s\n' "${end_marker}"
  } >> "${profile}"
fi

"${bin_path}" --version >/dev/null
echo "Codex CLI ${version} installed from offline DMG package."
