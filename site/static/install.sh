#!/bin/bash

REPO="logscore/drago"
INSTALL_DIR="/usr/local/bin"

# Check OS
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "Error: Drago currently only supports Linux based operating systems."
    exit 1
fi

# Check dependencies
command -v curl >/dev/null 2>&1 || { echo "Error: curl is required but not installed"; exit 1; }
command -v tar >/dev/null 2>&1 || { echo "Error: tar is required but not installed"; exit 1; }

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
    x86_64|amd64) ARCH="amd64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Get latest release
LATEST=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep "tag_name" | cut -d '"' -f 4)

# Download and extract
curl -sL "https://github.com/$REPO/releases/download/$LATEST/drago-${ARCH}.tar.gz" -o /tmp/drago.tar.gz
tar -xzf /tmp/drago.tar.gz -C /tmp/

# Install
sudo mv /tmp/drago "$INSTALL_DIR/"
sudo chmod +x "$INSTALL_DIR/drago"

# Clean up
rm /tmp/drago.tar.gz /tmp/drago

echo "Installed drago to $INSTALL_DIR"
