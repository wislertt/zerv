#!/bin/bash
set -e

REPO="wislertt/zerv"
BINARY_NAME="zerv"

get_latest_release() {
    curl --silent "https://api.github.com/repos/$REPO/releases/latest" |
        grep '"tag_name":' |
        sed -E 's/.*"([^"]+)".*/\1/'
}

detect_target() {
    local platform=$(uname -s)
    local arch=$(uname -m)

    case "$platform" in
        Linux*)
            case "$arch" in
                x86_64|amd64) echo "x86_64-unknown-linux-gnu" ;;
                aarch64|arm64) echo "aarch64-unknown-linux-gnu" ;;
                *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
            esac ;;
        Darwin*)
            case "$arch" in
                x86_64|amd64) echo "x86_64-apple-darwin" ;;
                aarch64|arm64) echo "aarch64-apple-darwin" ;;
                *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
            esac ;;
        CYGWIN*|MINGW*|MSYS*)
            case "$arch" in
                x86_64|amd64) echo "x86_64-pc-windows-msvc" ;;
                aarch64|arm64) echo "aarch64-pc-windows-msvc" ;;
                *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
            esac ;;
        *)
            echo "Unsupported platform: $platform" >&2; exit 1 ;;
    esac
}

main() {
    local target=$(detect_target)
    local version=${1:-${ZERV_VERSION:-$(get_latest_release)}}
    local asset_name="${BINARY_NAME}-${target}"

    if [[ "$target" == *"-windows-"* ]]; then
        asset_name="${asset_name}.exe"
    fi

    local download_url="https://github.com/$REPO/releases/download/$version/$asset_name"
    local install_dir="$HOME/.local/bin"
    local binary_path="$install_dir/$BINARY_NAME"

    echo "Installing $BINARY_NAME $version for $target..."
    echo "Download URL: $download_url"

    mkdir -p "$install_dir"

    if ! curl -L "$download_url" -o "$binary_path"; then
        echo "Error: Failed to download $asset_name" >&2
        exit 1
    fi

    # Check if downloaded file is actually a binary (not an error page)
    if [ ! -s "$binary_path" ] || file "$binary_path" | grep -q "text"; then
        echo "Error: Downloaded file is not a valid binary. Asset may not exist for $version" >&2
        rm -f "$binary_path"
        exit 1
    fi

    chmod +x "$binary_path"

    echo "$BINARY_NAME installed to $binary_path"
    echo "Make sure $install_dir is in your PATH"

    if ! echo "$PATH" | grep -q "$install_dir"; then
        echo "Add this to your shell profile:"
        echo "export PATH=\"\$HOME/.local/bin:\$PATH\""
    fi
}

main "$@"
