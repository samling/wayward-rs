FROM ubuntu:24.04

ARG GTK4_LAYER_SHELL_VERSION=1.3.0
ARG RUST_VERSION=1.93

ENV CARGO_HOME=/usr/local/cargo
ENV CARGO_TERM_COLOR=always
ENV PATH=/usr/local/cargo/bin:$PATH
ENV RUSTUP_HOME=/usr/local/rustup

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    git \
    gobject-introspection \
    libdbus-1-dev \
    libgirepository1.0-dev \
    libgtk-4-dev \
    libudev-dev \
    libwayland-dev \
    meson \
    ninja-build \
    pkg-config \
    wayland-protocols \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain "${RUST_VERSION}" \
    && chmod -R a+w "${CARGO_HOME}" "${RUSTUP_HOME}"

RUN git clone --depth 1 --branch "v${GTK4_LAYER_SHELL_VERSION}" \
    https://github.com/wmww/gtk4-layer-shell.git /tmp/gtk4-layer-shell \
    && meson setup \
        -Dexamples=false \
        -Ddocs=false \
        -Dtests=false \
        -Dsmoke-tests=false \
        -Dintrospection=false \
        -Dvapi=false \
        /tmp/gtk4-layer-shell/build \
        /tmp/gtk4-layer-shell \
    && ninja -C /tmp/gtk4-layer-shell/build \
    && ninja -C /tmp/gtk4-layer-shell/build install \
    && ldconfig \
    && rm -rf /tmp/gtk4-layer-shell

WORKDIR /workspace
