#!/usr/bin/env bash
set -euo pipefail

OWNER="logscore"
REPO="drago"
BIN="drago"
VERSION="${VERSION:-latest}"

ARCH=$(uname -m)
case "$ARCH" in
  x86_64) ARCH="amd64" ;;
  aarch64|arm64) ARCH="arm64" ;;
  *) echo "Unsupported arch: $ARCH" >&2; exit 1 ;;
esac

if [ "$VERSION" = "latest" ]; then
  TAG=$(curl -fsSL https://api.github.com/repos/${OWNER}/${REPO}/releases/latest | \
    grep -oE '"tag_name":\s*"[^"]+"' | cut -d'"' -f4)
else
  TAG="$VERSION"
fi

curl -fL https://github.com/${OWNER}/${REPO}/releases/download/${TAG}/${BIN}-${ARCH} -o /tmp/${BIN}
chmod +x /tmp/${BIN}
sudo install -m 0755 /tmp/${BIN} /usr/local/bin/${BIN}
rm /tmp/${BIN}
/usr/local/bin/${BIN} --help