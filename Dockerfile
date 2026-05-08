# Build stage
FROM rust:1.77-slim-bookworm AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
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
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the workspace
COPY . .

# Build the simulator in release mode
# We use --locked to ensure reproducible builds
RUN cargo build --release -p simulator

# Runtime stage (optional, for smaller image)
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libgtk-3-0 \
    libasound2 \
    libgmp10 \
    libmpfr6 \
    libcgal13 \
    libboost-system1.74.0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/simulator /app/dpm-simulator
COPY --from=builder /app/simulator/src/assets /app/assets

# Note: Running GUI apps from Docker requires X11/Wayland forwarding.
# This image is primarily intended for generating the binary or .deb package.
ENTRYPOINT ["/app/dpm-simulator"]
