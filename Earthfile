ARG package=enet-mqtt

###########################################################################
# BASE IMAGES
###########################################################################

# aarch64

base-aarch64-unknown-linux-gnu:
  FROM --platform linux/arm64 arm64v8/rust

  ENV DEBIAN_FRONTEND=noninteractive
  ENV target=aarch64-unknown-linux-gnu
  RUN apt-get update && apt-get install -yq libssl-dev git cmake lld

  WORKDIR /src
  COPY rust-toolchain.toml /src
  RUN cargo --version

base-aarch64-unknown-linux-musl:
  FROM --platform linux/arm64 arm64v8/rust:alpine

  ENV target=aarch64-unknown-linux-musl
  RUN apk add openssl-dev lld musl-dev perl make cmake

  WORKDIR /src
  COPY rust-toolchain.toml /src
  RUN cargo --version

# amd64

base-amd64-unknown-linux-gnu:
  FROM --platform linux/amd64 amd64/rust

  ENV DEBIAN_FRONTEND=noninteractive
  ENV target=x86_64-unknown-linux-gnu
  RUN apt-get update && apt-get install -yq libssl-dev git cmake lld

  WORKDIR /src
  COPY rust-toolchain.toml /src
  RUN cargo --version

base-amd64-unknown-linux-musl:
  FROM --platform linux/amd64 amd64/rust:alpine

  ENV target=x86_64-unknown-linux-musl
  RUN apk add openssl-dev lld musl-dev perl make cmake

  WORKDIR /src
  COPY rust-toolchain.toml /src
  RUN cargo --version

###########################################################################
# BUILD TARGETS
###########################################################################

# aarch64

build-aarch64-linux-gnu:
  FROM +base-aarch64-unknown-linux-gnu

  WORKDIR /src
  COPY --dir .cargo src Cargo.lock Cargo.toml /src

  RUN \
    --mount=type=cache,target=$HOME/.cargo/bin \
    --mount=type=cache,target=$HOME/.cargo/.crates2.json \
    --mount=type=cache,target=$HOME/.cargo/.crates.toml \
    --mount=type=cache,target=$HOME/.cargo/git \
    --mount=type=cache,target=$HOME/.cargo/registry/cache \
    --mount=type=cache,target=$HOME/.cargo/registry/index \
    --mount=type=cache,target=/src/target \
    cargo build --target ${target} --release --package ${package} --locked --bin ${package} \
    && mkdir -p /src/out \
    && cp /src/target/${target}/release/${package} /src/out/${package}

    SAVE ARTIFACT /src/out/${package}

build-aarch64-linux-gnu-vendored:
  FROM +base-aarch64-unknown-linux-gnu

  WORKDIR /src
  COPY --dir .cargo src Cargo.lock Cargo.toml /src

  RUN \
    --mount=type=cache,target=$HOME/.cargo/bin \
    --mount=type=cache,target=$HOME/.cargo/.crates2.json \
    --mount=type=cache,target=$HOME/.cargo/.crates.toml \
    --mount=type=cache,target=$HOME/.cargo/git \
    --mount=type=cache,target=$HOME/.cargo/registry/cache \
    --mount=type=cache,target=$HOME/.cargo/registry/index \
    --mount=type=cache,target=/src/target \
    cargo build --target ${target} --release --package ${package} --locked --bin ${package}  --features vendored \
    && mkdir -p /src/out \
    && cp /src/target/${target}/release/${package} /src/out/${package}

    SAVE ARTIFACT /src/out/${package}

build-aarch64-linux-musl-static:
  FROM +base-aarch64-unknown-linux-musl

  WORKDIR /src
  COPY --dir .cargo src Cargo.lock Cargo.toml /src

  RUN \
    --mount=type=cache,target=$HOME/.cargo/bin \
    --mount=type=cache,target=$HOME/.cargo/.crates2.json \
    --mount=type=cache,target=$HOME/.cargo/.crates.toml \
    --mount=type=cache,target=$HOME/.cargo/git \
    --mount=type=cache,target=$HOME/.cargo/registry/cache \
    --mount=type=cache,target=$HOME/.cargo/registry/index \
    --mount=type=cache,target=/src/target \
    cargo build --target ${target} --release --package ${package} --locked --bin ${package} --features vendored \
    && mkdir -p /src/out \
    && cp /src/target/${target}/release/${package} /src/out/${package}

    SAVE ARTIFACT /src/out/${package}

# amd64

build-amd64-linux-gnu:
  FROM +base-amd64-unknown-linux-gnu

  WORKDIR /src
  COPY --dir .cargo src Cargo.lock Cargo.toml /src

  RUN \
    --mount=type=cache,target=$HOME/.cargo/bin \
    --mount=type=cache,target=$HOME/.cargo/.crates2.json \
    --mount=type=cache,target=$HOME/.cargo/.crates.toml \
    --mount=type=cache,target=$HOME/.cargo/git \
    --mount=type=cache,target=$HOME/.cargo/registry/cache \
    --mount=type=cache,target=$HOME/.cargo/registry/index \
    --mount=type=cache,target=/src/target \
    cargo build --target ${target} --release --package ${package} --locked --bin ${package} \
    && mkdir -p /src/out \
    && cp /src/target/${target}/release/${package} /src/out/${package}

    SAVE ARTIFACT /src/out/${package}

build-amd64-linux-gnu-vendored:
  FROM +base-amd64-unknown-linux-gnu

  WORKDIR /src
  COPY --dir .cargo src Cargo.lock Cargo.toml /src

  RUN \
    --mount=type=cache,target=$HOME/.cargo/bin \
    --mount=type=cache,target=$HOME/.cargo/.crates2.json \
    --mount=type=cache,target=$HOME/.cargo/.crates.toml \
    --mount=type=cache,target=$HOME/.cargo/git \
    --mount=type=cache,target=$HOME/.cargo/registry/cache \
    --mount=type=cache,target=$HOME/.cargo/registry/index \
    --mount=type=cache,target=/src/target \
    cargo build --target ${target} --release --package ${package} --locked --bin ${package}  --features vendored \
    && mkdir -p /src/out \
    && cp /src/target/${target}/release/${package} /src/out/${package}

    SAVE ARTIFACT /src/out/${package}

build-amd64-linux-musl-static:
  FROM +base-amd64-unknown-linux-musl

  WORKDIR /src
  COPY --dir .cargo src Cargo.lock Cargo.toml /src

  RUN \
    --mount=type=cache,target=$HOME/.cargo/bin \
    --mount=type=cache,target=$HOME/.cargo/.crates2.json \
    --mount=type=cache,target=$HOME/.cargo/.crates.toml \
    --mount=type=cache,target=$HOME/.cargo/git \
    --mount=type=cache,target=$HOME/.cargo/registry/cache \
    --mount=type=cache,target=$HOME/.cargo/registry/index \
    --mount=type=cache,target=/src/target \
    cargo build --target ${target} --release --package ${package} --locked --bin ${package} --features vendored \
    && mkdir -p /src/out \
    && cp /src/target/${target}/release/${package} /src/out/${package}

    SAVE ARTIFACT /src/out/${package}


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

  SAVE ARTIFACT /out/* AS LOCAL build/

aarch64-linux-gnu-vendored:
  FROM +version
  ENV platform=aarch64-linux-gnu-vendored

  COPY +build-aarch64-linux-gnu-vendored/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/* AS LOCAL build/

aarch64-linux-musl-static:
  FROM +version
  ENV platform=aarch64-linux-musl-static

  COPY +build-aarch64-linux-musl-static/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/* AS LOCAL build/

i686-linux-gnu:
  FROM +version
  ENV platform=i686-linux-gnu

  COPY +build-i686-linux-gnu/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/* AS LOCAL build/

i686-linux-gnu-vendored:
  FROM +version
  ENV platform=i686-linux-gnu-vendored

  COPY +build-i686-linux-gnu-vendored/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/* AS LOCAL build/

amd64-linux-gnu:
  FROM +version
  ENV platform=amd64-linux-gnu

  COPY +build-amd64-linux-gnu/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/* AS LOCAL build/

amd64-linux-gnu-vendored:
  FROM +version
  ENV platform=amd64-linux-gnu-vendored

  COPY +build-amd64-linux-gnu-vendored/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/* AS LOCAL build/

amd64-linux-musl-static:
  FROM +version
  ENV platform=amd64-linux-musl-static

  COPY +build-amd64-linux-musl-static/${package} /out/
  RUN mv ${package} "${package}-v$(cat .version)-${platform}"
  RUN sha256sum "${package}-v$(cat .version)-${platform}" > "${package}-v$(cat .version)-${platform}".sha256.txt
  RUN rm .version

  SAVE ARTIFACT /out/* AS LOCAL build/

###########################################################################
# GROUP TARGETS
###########################################################################

aarch64:
  BUILD +aarch64-linux-gnu
  BUILD +aarch64-linux-gnu-vendored
  BUILD +aarch64-linux-musl-static

amd64:
  BUILD +amd64-linux-gnu
  BUILD +amd64-linux-gnu-vendored
  BUILD +amd64-linux-musl-static

all:
  BUILD +aarch64
  BUILD +amd64
