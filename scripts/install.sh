#!/bin/sh
# Install the kit binary on Linux (glibc) or macOS from a GitHub Release.
#   curl -fsSL https://raw.githubusercontent.com/Zwin-ux/kit/main/scripts/install.sh | sh
# The archive must match its SHA-256 line in SHA256SUMS. No sudo, no
# shell profile edits. Run with --help for options.
set -eu

error() { printf 'kit-install: error: %s\n' "$*" >&2; exit 1; }
note() { printf 'kit-install: %s\n' "$*"; }
fallback() { error "no prebuilt kit for $1. Build it from source: cargo install --git https://github.com/Zwin-ux/kit kitctl --locked"; }

download() {
    case $1 in
        https://*)
            if [ "$downloader" = curl ]; then
                curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSL -o "$2" "$1"
            else
                wget --https-only -q -O "$2" "$1"
            fi ;;
        http://127.0.0.1:*|http://localhost:*)
            if [ "$downloader" = curl ]; then
                curl -fsSL -o "$2" "$1"
            else
                wget -q -O "$2" "$1"
            fi ;;
        *) error "refusing non-HTTPS URL: $1" ;;
    esac
}

main() {
    version=${KIT_VERSION:-}
    dir=${KIT_INSTALL_DIR:-"$HOME/.local/bin"}
    default_base=https://github.com/Zwin-ux/kit/releases/download
    base=${KIT_DOWNLOAD_BASE:-$default_base}
    prerelease=0
    while [ "$#" -gt 0 ]; do
        case $1 in
            --version)
                [ "$#" -ge 2 ] || error '--version needs a value'
                version=$2
                shift 2 ;;
            --prerelease) prerelease=1; shift ;;
            --help)
                cat <<'HELP'
Usage: curl -fsSL https://raw.githubusercontent.com/Zwin-ux/kit/main/scripts/install.sh | sh
       curl -fsSL https://raw.githubusercontent.com/Zwin-ux/kit/main/scripts/install.sh | sh -s -- --version <version>
Options: --version <v>  --prerelease  --help
Env: KIT_VERSION, KIT_INSTALL_DIR (default ~/.local/bin),
     KIT_DOWNLOAD_BASE (for testing and mirrors; requires a version)
HELP
                return 0 ;;
            *) error "unknown option: $1" ;;
        esac
    done

    case $base in
        https://*|http://127.0.0.1:*|http://localhost:*) ;;
        *) error "refusing non-HTTPS KIT_DOWNLOAD_BASE: $base" ;;
    esac
    [ -n "$version" ] || [ "$base" = "$default_base" ] || error 'set KIT_VERSION or --version when KIT_DOWNLOAD_BASE is set'

    os=$(uname -s)
    arch=$(uname -m)
    case "$os:$arch" in
        Linux:x86_64|Linux:amd64) target=x86_64-unknown-linux-gnu ;;
        Linux:aarch64|Linux:arm64) target=aarch64-unknown-linux-gnu ;;
        Darwin:x86_64)
            target=x86_64-apple-darwin
            if [ "$(sysctl -n hw.optional.arm64 2>/dev/null || :)" = 1 ]; then
                target=aarch64-apple-darwin
            fi ;;
        Darwin:arm64) target=aarch64-apple-darwin ;;
        *) fallback "$os/$arch" ;;
    esac
    if [ "$os" = Linux ]; then
        if { command -v ldd >/dev/null 2>&1 && ldd --version 2>&1 | grep -qi musl; }; then
            fallback "$os/$arch (musl)"
        fi
        for musl_loader in /lib/ld-musl-*; do
            if [ -e "$musl_loader" ]; then fallback "$os/$arch (musl)"; fi
        done
    fi

    if command -v sha256sum >/dev/null 2>&1; then
        hash_tool=sha256sum
    elif command -v shasum >/dev/null 2>&1; then
        hash_tool=shasum
    else
        error 'need sha256sum or shasum'
    fi
    if command -v curl >/dev/null 2>&1; then
        downloader=curl
    elif command -v wget >/dev/null 2>&1; then
        downloader=wget
    else
        error 'need curl or wget'
    fi

    tmp=$(mktemp -d) || error 'could not create temporary directory'
    trap 'rm -rf "$tmp"' 0
    trap 'exit 1' 1 2 3 15
    if [ -z "$version" ]; then
        # 0.x releases are the old Node app and have no archives: skip them.
        api=https://api.github.com/repos/Zwin-ux/kit/releases/latest
        [ "$prerelease" -eq 0 ] || api='https://api.github.com/repos/Zwin-ux/kit/releases?per_page=30'
        download "$api" "$tmp/release.json" || error 'could not find the latest release; use --version'
        version=$(tr ',' '\n' < "$tmp/release.json" |
            sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' |
            awk '!/^v?0\./ { print; exit }')
        if [ -z "$version" ]; then
            if [ "$prerelease" -eq 0 ]; then
                error 'no stable release of kit yet; use --prerelease or --version'
            fi
            error 'no kit release found; use --version'
        fi
    fi
    version=${version#v}
    case $version in
        ''|*[!A-Za-z0-9._-]*) error "invalid version: $version" ;;
    esac

    archive=kit-$version-$target.tar.gz
    url=${base%/}/v$version
    note "downloading $url/$archive"
    download "$url/$archive" "$tmp/$archive" || error "could not download $archive; check that release v$version exists"
    download "$url/SHA256SUMS" "$tmp/SHA256SUMS" || error 'could not download SHA256SUMS'
    expected=$(awk -v file="$archive" '$2 == file || $2 == "*" file { print $1; exit }' "$tmp/SHA256SUMS")
    [ -n "$expected" ] || error "SHA256SUMS has no entry for $archive"
    if [ "$hash_tool" = sha256sum ]; then
        actual=$(sha256sum "$tmp/$archive" | awk '{ print $1 }')
    else
        actual=$(shasum -a 256 "$tmp/$archive" | awk '{ print $1 }')
    fi
    expected=$(printf '%s' "$expected" | tr 'A-F' 'a-f')
    actual=$(printf '%s' "$actual" | tr 'A-F' 'a-f')
    if [ "$actual" != "$expected" ]; then
        printf 'kit-install: expected SHA256: %s\nkit-install: actual SHA256: %s\n' "$expected" "$actual" >&2
        error 'checksum mismatch; refusing to install'
    fi
    tar -xzf "$tmp/$archive" -C "$tmp" || error 'could not extract archive'
    binary=$tmp/kit-$version-$target/kit
    [ -f "$binary" ] || error "archive has no kit binary at $binary"

    mkdir -p "$dir" || error 'cannot create install directory; set KIT_INSTALL_DIR to a writable dir'
    staged=$dir/.kit.tmp.$$
    cp "$binary" "$staged" || error 'cannot copy kit; set KIT_INSTALL_DIR to a writable dir'
    chmod 755 "$staged" || error 'cannot make kit executable; set KIT_INSTALL_DIR to a writable dir'
    mv -f "$staged" "$dir/kit" || { rm -f "$staged"; error 'cannot install kit; set KIT_INSTALL_DIR to a writable dir'; }
    note "installed kit $version to $dir/kit"
    if installed_version=$("$dir/kit" --version 2>&1); then
        note "$installed_version"
    else
        printf 'kit-install: warning: kit --version failed\n' >&2
    fi
    case :$PATH: in
        *":$dir:"*) ;;
        *) printf "kit-install: warning: add this line to ~/.profile, ~/.bashrc or ~/.zshrc:\nexport PATH=\"%s:\$PATH\"\n" "$dir" ;;
    esac
    note 'next: kit setup'
}

main "$@"
