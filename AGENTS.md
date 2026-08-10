# OpenEQ contributor guide

OpenEQ is a real-time, stereo, three-band equalizer plugin written in Rust. It exports CLAP and
VST3 through `nice-plug`; the DSP is deliberately independent from the plugin framework.

## Repository layout

- `src/dsp/` contains the pure filter math. Keep it deterministic, allocation-free, and covered by
  unit tests.
- `src/lib.rs` owns stable parameter IDs and the `nice-plug` host integration. Treat IDs and plugin
  identifiers as public compatibility contracts once a release ships.
- `src/editor/` contains the custom egui editor. `build_ui()` is shared by the plugin and
  `examples/editor_preview.rs`; do not create a preview-only copy of the layout.
- `xtask/` exposes the upstream plugin bundling command.

## Audio-thread rules

- `process()` and all code it calls must not allocate, lock, log, block, or perform file/network
  I/O.
- Advance smoothed parameters once per sample frame, then apply the same values to both stereo
  channels. Never advance a parameter smoother per channel.
- Recalculate coefficients only when their band settings have changed, and preserve each filter's
  delay state while coefficients change.
- Keep denormal/NaN behavior intentional. Coefficients must be finite for every exposed parameter
  value, including boundary values.

## Editor rules

- Bind real editor controls through `ParamSetter`; never mutate a parameter value directly.
- UI work, including the response curve's allocations and coefficient calculations, belongs on the
  GUI thread only. The audio callback must not depend on the editor being open.
- Test the standalone preview with `cargo run --example editor_preview`. It uses the same native
  baseview surface as the DAW editor, which is especially important when changing macOS UI code.

## Development workflow

Run these before handing off a change:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Build distributable plugin bundles with:

```sh
cargo xtask bundle openeq --release
```

## Scope discipline

The initial product is a stereo three-band EQ: low shelf, mid bell, and high shelf. Additions such
as a custom editor, spectrum analyzer, oversampling, extra bands, M/S processing, or new plugin
formats are product decisions; discuss them before widening the public surface.
