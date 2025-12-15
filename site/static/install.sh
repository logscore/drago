#!/bin/sh
set -e

REPO="logscore/drago"
INSTALL_DIR="/usr/local/bin"

echo "Starting Drago installation..."

# Check OS
echo "Checking operating system..."
if [ ! -f /etc/os-release ]; then
  echo "Error: /etc/os-release not found"
  exit 1
fi

if ! grep -qi "debian\|ubuntu" /etc/os-release; then
  echo "Error: Drago currently only supports Debian-based Linux distributions."
  cat /etc/os-release
  exit 1
fi
echo "OS check passed"

# Check dependencies
echo "Checking dependencies..."
if ! command -v curl >/dev/null 2>&1; then
  echo "Error: curl is required but not installed"
  exit 1
fi
echo "curl found"

if ! command -v tar >/dev/null 2>&1; then
  echo "Error: tar is required but not installed"
  exit 1
fi
echo "tar found"

# Detect architecture
echo "Detecting architecture..."
ARCH=$(uname -m)
echo "Detected architecture: $ARCH"
case "$ARCH" in
  x86_64|amd64) ARCH="amd64" ;;
  aarch64|arm64) ARCH="arm64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac
echo "Using architecture: $ARCH"

# Get latest release
echo "Fetching latest release information..."
LATEST=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep "tag_name" | cut -d '"' -f 4)
if [ -z "$LATEST" ]; then
  echo "Error: Failed to fetch latest release information"
  exit 1
fi
echo "Latest release: $LATEST"

# Download and extract
echo "Downloading Drago $LATEST for $ARCH..."
curl -sL "https://github.com/$REPO/releases/download/$LATEST/drago-${ARCH}.tar.gz" -o /tmp/drago.tar.gz
if [ ! -f /tmp/drago.tar.gz ]; then
  echo "Error: Failed to download Drago"
  exit 1
fi
echo "Download completed"

echo "Extracting archive..."
tar -xzf /tmp/drago.tar.gz -C /tmp/
if [ ! -f /tmp/drago ]; then
  echo "Error: Failed to extract Drago"
  exit 1
fi
echo "Extraction completed"

# Install
echo "Installing Drago to $INSTALL_DIR..."
sudo mv /tmp/drago "$INSTALL_DIR/"
sudo chmod +x "$INSTALL_DIR/drago"
echo "Installation completed"

# Clean up
echo "Cleaning up temporary files..."
rm /tmp/drago.tar.gz /tmp/drago
echo "Cleanup completed"

echo "Installed drago to $INSTALL_DIR"
echo "Installation successful! You can now run 'drago' from anywhere."
