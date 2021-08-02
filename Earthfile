ARG package=enet-mqtt

FROM rust
RUN apt-get update \
  && apt-get install -y libudev-dev cmake gcc-aarch64-linux-gnu \
  && rm -rf /var/lib/apt/lists/*

all:
  BUILD +build-amd64
  BUILD +build-aarch64

build-amd64:
  # x86_64-unknown-linux-gnu
  BUILD \
    --build-arg target=x86_64-unknown-linux-gnu \
    --build-arg platform=x86_64 \
    +build

build-aarch64:
  # aarch64-unknown-linux-gnu
  BUILD \
    --build-arg target=aarch64-unknown-linux-gnu \
    --build-arg platform=aarch64 \
    +build

build:
  ARG target=x86_64-unknown-linux-gnu
  ARG platform=x86_64

  RUN rustup target add ${target}

  COPY Cargo.lock /src/Cargo.lock
  WORKDIR /src
  # Update index
  RUN --mount=type=cache,target=/src/target cargo install lazy_static >/dev/null 2>/dev/null || true

  COPY . /src
  RUN --mount=type=cache,target=/src/target cargo build --target ${target} --release --package ${package} --locked --bin ${package} \
    && cp /src/target/${target}/release/${package} /src/${package}

  SAVE ARTIFACT /src/${package} AS LOCAL build/${package}-${platform}
