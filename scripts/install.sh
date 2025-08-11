#!/bin/bash
set -e

REPO="wisarootl/zerv"
BINARY_NAME="zerv"

get_latest_release() {
    curl --silent "https://api.github.com/repos/$REPO/releases/latest" |
        grep '"tag_name":' |
        sed -E 's/.*"([^"]+)".*/\1/'
}

detect_platform() {
    local platform
    case "$(uname -s)" in
        Linux*)     platform="linux" ;;
        Darwin*)    platform="macos" ;;
        CYGWIN*|MINGW*|MSYS*) platform="windows" ;;
        *)          echo "Unsupported platform: $(uname -s)" >&2; exit 1 ;;
    esac
    echo "$platform"
}

detect_arch() {
    local arch
    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="arm64" ;;
        *)          echo "Unsupported architecture: $(uname -m)" >&2; exit 1 ;;
    esac
    echo "$arch"
}

main() {
    local platform=$(detect_platform)
    local arch=$(detect_arch)
    local version=${1:-${ZERV_VERSION:-$(get_latest_release)}}
    local asset_name="${BINARY_NAME}-${platform}-${arch}"

    if [ "$platform" = "windows" ]; then
        asset_name="${asset_name}.exe"
    fi

    local download_url="https://github.com/$REPO/releases/download/$version/$asset_name"
    local install_dir="$HOME/.local/bin"
    local binary_path="$install_dir/$BINARY_NAME"

    echo "Installing $BINARY_NAME $version for $platform..."
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
