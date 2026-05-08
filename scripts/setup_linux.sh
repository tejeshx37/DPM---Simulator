#!/bin/bash

# DPM Simulator - Linux Setup Script
# This script installs all necessary system dependencies for building and running the simulator.

set -e

echo "--------------------------------------------------"
echo "DPM Simulator - Linux Dependency Setup"
echo "--------------------------------------------------"

if [ "$EUID" -ne 0 ]; then
  echo "Please run as root (use sudo)"
  exit 1
fi

echo "[1/2] Updating package lists..."
apt-get update

echo "[2/2] Installing dependencies..."
apt-get install -y \
    build-essential \
    pkg-config \
    libcgal-dev \
    libboost-all-dev \
    libgmp-dev \
    libmpfr-dev \
    libgtk-3-dev \
    libasound2-dev \
    libvulkan-dev \
    libwayland-dev \
    libxkbcommon-dev \
    cmake

echo "--------------------------------------------------"
echo "Setup Complete!"
echo "You can now build the project using: cargo build --release"
echo "--------------------------------------------------"
