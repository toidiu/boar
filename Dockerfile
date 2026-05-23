# syntax=docker/dockerfile:1.7

# Rust 1.85+ is required for edition 2024 (see Cargo.toml). trixie ships
# fresh cmake / perl / go, which BoringSSL (built by deps/quiche's build
# script) is sensitive to.
FROM rust:1-trixie

RUN apt-get update && apt-get install -y --no-install-recommends \
        iproute2 \
        ethtool \
        iputils-ping \
        tcpdump \
        sudo \
        ca-certificates \
        cmake \
        clang \
        pkg-config \
        libssl-dev \
        perl \
        golang \
        git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /boar

COPY . .

# build.rs shells out to `cargo build` inside deps/quiche, so the submodule
# must be populated in the build context.
RUN test -f deps/quiche/Cargo.toml \
    || { echo 'deps/quiche is empty — run `git submodule update --init` before `docker build`'; exit 1; }

# Cache mounts: registry/git avoid re-fetching crates; the two target dirs
# avoid recompiling unchanged crates (BoringSSL is the expensive one). Cache
# mounts vanish after the RUN, so we copy the artefacts we need into
# /opt/boar/bin, which persists in the image layer.
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/boar/target,id=boar-target,sharing=locked \
    --mount=type=cache,target=/boar/deps/quiche/target,id=quiche-target,sharing=locked \
    cargo build --bin boar \
 && install -D target/debug/boar /opt/boar/bin/boar \
 && install -D deps/quiche/target/debug/quiche-client /opt/boar/bin/quiche-client \
 && install -D deps/quiche/target/debug/examples/async_http3_server /opt/boar/bin/async_http3_server

# boar's source (endpoint.rs, args.rs) and the compose command both reference
# the binaries at their cargo-relative paths. The cache mount wiped those
# locations, so re-create them as symlinks into /opt/boar/bin.
RUN mkdir -p target/debug deps/quiche/target/debug/examples \
 && ln -sf /opt/boar/bin/boar target/debug/boar \
 && ln -sf /opt/boar/bin/quiche-client deps/quiche/target/debug/quiche-client \
 && ln -sf /opt/boar/bin/async_http3_server deps/quiche/target/debug/examples/async_http3_server

ENV PATH="/opt/boar/bin:${PATH}"

ENTRYPOINT ["/boar/scripts/docker_entrypoint.sh"]
CMD ["boar"]
