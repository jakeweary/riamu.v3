# Riamu v3 – autism unleashed

Third iteration of my Discord bot, this time with an unhealthy level of overengineering.

## Instructions

### Configuration

Rename `.env.example` to `.env` and fill it with actual values.

### Running in prod

```sh
scripts/get-assets

docker compose up -d # launch
docker compose logs -f # check logs
docker compose down # shutdown
```

### Development

#### Preparing dev environment

```sh
sudo apt update
sudo apt install -y \
  python3 python3-pip ffmpeg sqlite3 curl git build-essential pkg-config clang \
  python3-dev llvm-dev libclang-dev libssl-dev libpango1.0-dev libcairo2-dev librsvg2-dev

curl -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
rustup toolchain install nightly

python3 -m pip install --user pipenv
pipenv install --dev

scripts/get-assets
scripts/get-deps
```

#### Building / Running

```sh
pipenv run cargo run # build and run in debug mode
pipenv run cargo run --release # build and run in release mode

pipenv run cargo build # build in debug mode
pipenv run target/debug/riamu # run the debug build w/o cargo

pipenv run cargo build --release # build in release mode
pipenv run target/release/riamu # run the release build w/o cargo
```

#### Testing

```sh
pipenv run cargo test --workspace # rust
pipenv run pytest -v -n auto --dist loadscope python # python
```

#### Linting with clippy

```sh
cargo clippy --workspace # just lint
cargo clippy --workspace --fix --allow-dirty --allow-staged --broken-code # lint & fix
```

### Misc.

```sh
# acquire prod database over ssh
ssh foo@bar.baz 'docker cp riamu.v3:/app/data - | gzip' | tar -xzv --strip-components=1

# query local database
sqlite3 -box -cmd '.load deps/sqlean' db.sqlite

# query dockerized database
docker run --volumes-from riamu.v3 --rm -it alpine \
  sh -c 'apk add sqlite && sqlite3 -box /app/data/db.sqlite'
```

## Roadmap

### Alpha
- [x] Python interop
- [x] A proc macro for slash commands
- [x] Decent enough error handling with unique error ids

### Beta
- [x] Subcommands support
- [x] Respect attachment size limits
- [x] File storage system for large files
- [ ] Core commands basic functionality
  - [ ] `/download` – yt-dlp wrapper
  - [x] `/deezer` – deezer downloading
  - [x] `/tiktok` – tiktok downloading

### Release
- [ ] More commands
- [ ] Rate limits, permissions
- [ ] Improved error handling
  - [ ] All common cases handled gracefully
  - [x] Using ephemeral messages
  - [ ] Using `anyhow::Context` or similar

### Post-release
- [ ] Context menus, auto-complete, etc.
- [ ] Some way to support Deezer/Spotify playlists
- [ ] gallery-dl
