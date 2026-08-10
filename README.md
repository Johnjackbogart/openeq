# OpenEQ

OpenEQ is an open-source, real-time-safe three-band stereo equalizer plugin written in Rust. It
exports VST3 and CLAP through [`nice-plug`](https://codeberg.org/RustAudio/nice-plug), a
community-led successor to `nih-plug`.

The initial topology is deliberately focused:

- Low shelf: frequency and gain
- Mid bell: frequency, gain, and Q
- High shelf: frequency and gain

The plugin includes a custom native editor and still works with a host's generic parameter view.
An analyzer, additional bands, M/S mode, oversampling, and AU support are intentionally deferred
until the core DSP is validated.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Build plugin bundles

```sh
cargo xtask bundle openeq --release
```

The bundle command writes `target/bundled/openeq.vst3` and `target/bundled/openeq.clap`. Copy the
appropriate output to your DAW's plugin directory and rescan plugins. On macOS, VST3 plugins
normally go in `~/Library/Audio/Plug-Ins/VST3/`.

## Preview the custom editor

Run the standalone native editor without a DAW:

```sh
cargo run --example editor_preview
```

The preview uses the same egui/baseview layout as the plugin editor, with local in-memory controls
and an animated response curve. It is the quickest way to iterate on the UI before testing in a
DAW.

## Architecture

Filter math lives in `src/dsp/` and is unit-tested without a DAW/plugin host. `src/lib.rs` contains
only parameter and plugin-format integration. See [AGENTS.md](AGENTS.md) for the real-time and
contribution rules.

## License

MIT — see [LICENSE](LICENSE).
