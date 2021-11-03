ARG package=enet-mqtt
FROM busybox

###########################################################################
# BASE IMAGES
###########################################################################

# aarch64

base-aarch64-unknown-linux-gnu:
  FROM --platform linux/arm64 arm64v8/rust

  ENV DEBIAN_FRONTEND=noninteractive
  ENV RUST_BACKTRACE=1
  ENV target=aarch64-unknown-linux-gnu
  RUN apt-get update && apt-get install -yq libssl-dev git cmake lld

  WORKDIR /src
  COPY rust-toolchain.toml /src
  RUN cargo --version
  WORKDIR /src
  SAVE IMAGE --cache-hint

base-aarch64-unknown-linux-musl:
  FROM --platform linux/arm64 arm64v8/rust:alpine

  ENV RUST_BACKTRACE=1
  ENV target=aarch64-unknown-linux-musl
  RUN apk add openssl-dev lld musl-dev perl make cmake

  WORKDIR /src
  COPY rust-toolchain.toml /src
  RUN cargo --version
  WORKDIR /src
  SAVE IMAGE --cache-hint

# amd64

base-amd64-unknown-linux-gnu:
  FROM --platform linux/amd64 amd64/rust

  ENV DEBIAN_FRONTEND=noninteractive
  ENV RUST_BACKTRACE=1
  ENV target=x86_64-unknown-linux-gnu
  RUN apt-get update && apt-get install -yq libssl-dev git cmake lld

  WORKDIR /src
  COPY rust-toolchain.toml /src
  RUN cargo --version
  WORKDIR /src
  SAVE IMAGE --cache-hint

base-amd64-unknown-linux-musl:
  FROM --platform linux/amd64 amd64/rust:alpine

  ENV RUST_BACKTRACE=1
  ENV target=x86_64-unknown-linux-musl
  RUN apk add openssl-dev lld musl-dev perl make cmake

  WORKDIR /src
  COPY rust-toolchain.toml /src
  RUN cargo --version
  WORKDIR /src
  SAVE IMAGE --cache-hint

base-all:
  BUILD +base-aarch64-unknown-linux-gnu
  BUILD +base-aarch64-unknown-linux-musl
  BUILD +base-amd64-unknown-linux-gnu
  BUILD +base-amd64-unknown-linux-musl

###########################################################################
# CHEF TARGETS
###########################################################################

# aarch64

chef-aarch64-unknown-linux-gnu:
  FROM +base-aarch64-unknown-linux-gnu
  RUN cargo install cargo-chef
  SAVE IMAGE --cache-hint

chef-aarch64-unknown-linux-musl:
  FROM +base-aarch64-unknown-linux-musl
  RUN cargo install cargo-chef
  SAVE IMAGE --cache-hint

# amd64

chef-amd64-unknown-linux-gnu:
  FROM +base-amd64-unknown-linux-gnu
  RUN cargo install cargo-chef
  SAVE IMAGE --cache-hint

chef-amd64-unknown-linux-musl:
  FROM +base-amd64-unknown-linux-musl
  RUN cargo install cargo-chef
  SAVE IMAGE --cache-hint

chef-all:
  BUILD +chef-aarch64-unknown-linux-gnu
  BUILD +chef-aarch64-unknown-linux-musl
  BUILD +chef-amd64-unknown-linux-gnu
  BUILD +chef-amd64-unknown-linux-musl

###########################################################################
# PLAN TARGETS
###########################################################################

plan-aarch64-unknown-linux-gnu:
  FROM +chef-aarch64-unknown-linux-gnu
  COPY --dir .cargo src Cargo.lock Cargo.toml /src
  RUN cargo chef prepare --recipe-path recipe.json
  SAVE ARTIFACT recipe.json
  SAVE IMAGE --cache-hint

plan-aarch64-unknown-linux-musl:
  FROM +chef-aarch64-unknown-linux-musl
  COPY --dir .cargo src Cargo.lock Cargo.toml /src
  RUN cargo chef prepare --recipe-path recipe.json
  SAVE ARTIFACT recipe.json
  SAVE IMAGE --cache-hint

# amd64

plan-amd64-unknown-linux-gnu:
  FROM +chef-amd64-unknown-linux-gnu
  COPY --dir .cargo src Cargo.lock Cargo.toml /src
  RUN cargo chef prepare --recipe-path recipe.json
  SAVE ARTIFACT recipe.json
  SAVE IMAGE --cache-hint

plan-amd64-unknown-linux-musl:
  FROM +chef-amd64-unknown-linux-musl
  COPY --dir .cargo src Cargo.lock Cargo.toml /src
  RUN cargo chef prepare --recipe-path recipe.json
  SAVE ARTIFACT recipe.json
  SAVE IMAGE --cache-hint

plan-all:
  BUILD +plan-aarch64-unknown-linux-gnu
  BUILD +plan-aarch64-unknown-linux-musl
  BUILD +plan-amd64-unknown-linux-gnu
  BUILD +plan-amd64-unknown-linux-musl

###########################################################################
# DEPS TARGETS
###########################################################################

deps-aarch64-linux-gnu:
  FROM +chef-aarch64-unknown-linux-gnu
  COPY +plan-aarch64-unknown-linux-gnu/recipe.json .
  RUN cargo chef cook --recipe-path recipe.json --target ${target} --release --package ${package}
  SAVE IMAGE --cache-hint

deps-aarch64-linux-gnu-vendored:
  FROM +deps-aarch64-linux-gnu
  COPY +plan-aarch64-unknown-linux-gnu/recipe.json .
  RUN cargo chef cook --recipe-path recipe.json --target ${target} --release --package ${package} --features vendored
  SAVE IMAGE --cache-hint

deps-aarch64-linux-musl-static:
  FROM +chef-aarch64-unknown-linux-musl
  COPY +plan-aarch64-unknown-linux-musl/recipe.json .
  RUN cargo chef cook --recipe-path recipe.json --target ${target} --release --package ${package} --features vendored
  SAVE IMAGE --cache-hint

# amd64

deps-amd64-linux-gnu:
  FROM +chef-amd64-unknown-linux-gnu
  COPY +plan-amd64-unknown-linux-gnu/recipe.json .
  RUN cargo chef cook --recipe-path recipe.json --target ${target} --release --package ${package}
  SAVE IMAGE --cache-hint

deps-amd64-linux-gnu-vendored:
  FROM +deps-amd64-linux-gnu
  COPY +plan-amd64-unknown-linux-gnu/recipe.json .
  RUN cargo chef cook --recipe-path recipe.json --target ${target} --release --package ${package} --features vendored
  SAVE IMAGE --cache-hint

deps-amd64-linux-musl-static:
  FROM +chef-amd64-unknown-linux-musl
  COPY +plan-amd64-unknown-linux-musl/recipe.json .
  RUN cargo chef cook --recipe-path recipe.json --target ${target} --release --package ${package} --features vendored
  SAVE IMAGE --cache-hint

deps-all:
  BUILD +deps-aarch64-linux-gnu
  BUILD +deps-aarch64-linux-gnu-vendored
  BUILD +deps-aarch64-linux-musl-static
  BUILD +deps-amd64-linux-gnu
  BUILD +deps-amd64-linux-gnu-vendored
  BUILD +deps-amd64-linux-musl-static

###########################################################################
# BUILD TARGETS
###########################################################################

build-aarch64-linux-gnu:
  FROM +deps-aarch64-linux-gnu
  COPY --dir .cargo src Cargo.lock Cargo.toml /src
  RUN cargo build --target ${target} --release --package ${package} --locked --bin ${package}
  SAVE ARTIFACT target/${target}/release/${package}

build-aarch64-linux-gnu-vendored:
  FROM +deps-aarch64-linux-gnu-vendored
  COPY --dir .cargo src Cargo.lock Cargo.toml /src
  RUN cargo build --target ${target} --release --package ${package} --locked --bin ${package} --features vendored
  SAVE ARTIFACT target/${target}/release/${package}

build-aarch64-linux-musl-static:
  FROM +deps-aarch64-linux-musl-static
  COPY --dir .cargo src Cargo.lock Cargo.toml /src
  RUN cargo build --target ${target} --release --package ${package} --locked --bin ${package} --features vendored
  SAVE ARTIFACT target/${target}/release/${package}

# amd64

build-amd64-linux-gnu:
  FROM +deps-amd64-linux-gnu
  COPY --dir .cargo src Cargo.lock Cargo.toml /src
  RUN cargo build --target ${target} --release --package ${package} --locked --bin ${package}
  SAVE ARTIFACT target/${target}/release/${package}

build-amd64-linux-gnu-vendored:
  FROM +deps-amd64-linux-gnu-vendored
  COPY --dir .cargo src Cargo.lock Cargo.toml /src
  RUN cargo build --target ${target} --release --package ${package} --locked --bin ${package} --features vendored
  SAVE ARTIFACT target/${target}/release/${package}

build-amd64-linux-musl-static:
  FROM +deps-amd64-linux-musl-static
  COPY --dir .cargo src Cargo.lock Cargo.toml /src
  RUN cargo build --target ${target} --release --package ${package} --locked --bin ${package} --features vendored
  SAVE ARTIFACT target/${target}/release/${package}

###########################################################################
# VERSION HELPER
###########################################################################

version:
  FROM rust

  WORKDIR /src
  COPY --dir .cargo src Cargo.lock Cargo.toml rust-toolchain.toml /src
  RUN mkdir -p "/out" && cargo pkgid | cut -d# -f2 | cut -d: -f2 > /out/.version

  WORKDIR /out
  RUN echo "version=$(cat .version)"

###########################################################################
# ARTIFACT TARGETS
###########################################################################

aarch64-linux-gnu:
  FROM +version
  ENV platform=aarch64-linux-gnu

  COPY +build-aarch64-linux-gnu/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/*

aarch64-linux-gnu-vendored:
  FROM +version
  ENV platform=aarch64-linux-gnu-vendored

  COPY +build-aarch64-linux-gnu-vendored/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/*

aarch64-linux-musl-static:
  FROM +version
  ENV platform=aarch64-linux-musl-static

  COPY +build-aarch64-linux-musl-static/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/*

i686-linux-gnu:
  FROM +version
  ENV platform=i686-linux-gnu

  COPY +build-i686-linux-gnu/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/*

i686-linux-gnu-vendored:
  FROM +version
  ENV platform=i686-linux-gnu-vendored

  COPY +build-i686-linux-gnu-vendored/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/*

amd64-linux-gnu:
  FROM +version
  ENV platform=amd64-linux-gnu

  COPY +build-amd64-linux-gnu/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/*

amd64-linux-gnu-vendored:
  FROM +version
  ENV platform=amd64-linux-gnu-vendored

  COPY +build-amd64-linux-gnu-vendored/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/*

amd64-linux-musl-static:
  FROM +version
  ENV platform=amd64-linux-musl-static

  COPY +build-amd64-linux-musl-static/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/*

###########################################################################
# GROUP TARGETS
###########################################################################

aarch64:
  COPY +aarch64-linux-gnu/* /out/
  COPY +aarch64-linux-gnu-vendored/* /out/
  COPY +aarch64-linux-musl-static/* /out/

  SAVE ARTIFACT /out/*

amd64:
  COPY +amd64-linux-gnu/* /out/
  COPY +amd64-linux-gnu-vendored/* /out/
  COPY +amd64-linux-musl-static/* /out/

  SAVE ARTIFACT /out/*

all:
  COPY +aarch64/* /out/
  COPY +amd64/* /out/

  SAVE ARTIFACT /out/*
