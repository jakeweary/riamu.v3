FROM ubuntu:22.04 AS base

ARG DEBIAN_FRONTEND=noninteractive

ENV LANG=C.UTF-8

# ---

FROM base AS build

WORKDIR /build

RUN apt update && \
  apt install -y --no-install-recommends \
    ca-certificates curl git unzip build-essential

# ---

FROM build AS build-deps

COPY scripts/get-deps .
RUN ./get-deps

# ---

FROM build AS build-app

RUN apt update && \
  apt install -y --no-install-recommends \
    ca-certificates curl git build-essential pkg-config clang \
    python3-dev llvm-dev libclang-dev libssl-dev \
    libpango1.0-dev libcairo2-dev librsvg2-dev

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

RUN apt update && \
  apt install -y --no-install-recommends \
    python3 python3-pip python3-dev sqlite3 ffmpeg && \
  pip3 install pipenv && \
  apt autopurge -y python3-pip

COPY Pipfile* .
RUN pipenv install --deploy --system

COPY --from=build-deps /build/deps deps
COPY --from=build-app /build/riamu .
CMD ["./riamu"]
