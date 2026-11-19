# clik-color-filters

A [clik](https://clik.aiocean.io) plugin with three color filters for screenshots. Each filter changes only the visible part of every image in scope and keeps alpha.

## Steps

| Step | Role | What it does |
|---|---|---|
| `color-filters/grayscale` | filter | Rec. 601 luma gray |
| `color-filters/sepia` | filter | Warm brown sepia tone |
| `color-filters/invert` | filter | Inverts red, green and blue |

No props, no config, no network access.

## Install

In clik, open Settings → Plugins and either find it under **Browse**, or choose **Install from GitHub** and enter `aiocean/clik-color-filters`.

## Build

Needs the `wasm32-unknown-unknown` target and `wasm-tools`.

```bash
cargo test
cargo build --release --target wasm32-unknown-unknown
mkdir -p pkg
wasm-tools component new target/wasm32-unknown-unknown/release/clik_color_filters.wasm -o pkg/plugin.wasm
cp plugin.toml pkg/
```

Pushing a `v*` tag matching `version` in `plugin.toml` publishes `color-filters-<version>.clikplugin.tar.gz` as a GitHub release (`.github/workflows/release.yml`).

## License

MIT
