FROM ubuntu:22.04 AS base

ARG DEBIAN_FRONTEND=noninteractive

# ---

FROM base AS build
WORKDIR /build

RUN apt update && \
  apt install -y --no-install-recommends \
    ca-certificates curl git build-essential pkg-config clang \
    python3-dev llvm-dev libclang-dev libssl-dev libpango1.0-dev libcairo2-dev librsvg2-dev

RUN --mount=type=cache,target=/root/.rustup \
  --mount=type=cache,target=/root/.cargo \
  curl -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none && \
  ~/.cargo/bin/rustup toolchain install nightly --profile minimal

COPY . .
RUN --mount=type=cache,target=/root/.rustup \
  --mount=type=cache,target=/root/.cargo \
  --mount=type=cache,target=target \
  ~/.cargo/bin/cargo build --release && \
  mv target/release/riamu . && \
  strip riamu

# ---

FROM base AS app
WORKDIR /app

ARG PIP_DISABLE_PIP_VERSION_CHECK=1
ARG PIP_NO_CACHE_DIR=1

ENV LANG=C.UTF-8

RUN apt update && \
  apt install -y --no-install-recommends \
    python3 python3-pip python3-dev ffmpeg && \
  pip3 install pipenv && \
  apt autopurge -y python3-pip && \
  rm -rf /var/lib/apt/lists/*

COPY Pipfile* .
RUN pipenv install --deploy --system

COPY --from=build /build/riamu .
CMD ["./riamu"]
