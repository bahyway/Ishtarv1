FROM debian:bookworm-slim

# 1. Install System Dependencies as ROOT
# These are required to build Postgres extensions and Rust crates
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    pkg-config \
    libssl-dev \
    llvm-dev \
    libclang-dev \
    clang \
    cmake \
    libreadline-dev \
    zlib1g-dev \
    bison \
    flex \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 2. Create the Sovereign User and Workspace
RUN useradd -m akkad && \
    mkdir -p /workspace && \
    chown -R akkad:akkad /workspace

# 3. Switch to the AKKAD user for the rest of the installation
USER akkad
WORKDIR /home/akkad

# 4. Install Rust Toolchain AS THE USER
# This ensures all binaries belong to 'akkad'
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/home/akkad/.cargo/bin:${PATH}"

# 5. Add Rust Components for Advanced Intelligence
RUN rustup component add rust-analyzer rust-src clippy && \
    rustup target add wasm32-unknown-unknown

# 6. Install BDBWay Specific Tools (pgrx and wasm-pack)
# We use --locked to ensure version stability for the 1B node engine
RUN cargo install --locked cargo-pgrx --version 0.11.3
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# 7. Final Working Directory
WORKDIR /workspace

# Note: Once the container starts, remember to run 'cargo pgrx init'
