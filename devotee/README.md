# devotee
A bit personal visualization engine.

`devotee` is directly based on:
- [winit](https://crates.io/crates/winit) - Rust windowing library;
- [pixels](https://crates.io/crates/pixels) - `wgpu`-based pixel renderer;
- [rodio](https://crates.io/crates/rodio) - `cpal`-based audio playback library;

## Goals
`devotee` aims to provide __simple__ pixel-perfect visualization and optional sound effects.

## Non-goals
`devotee` does not aim to provide
- ECS architecture;
- resource loading;
- scripting;

## Work in progress
`devotee` is totally a work in progress.
We'd suggest to avoid relying on it in a long term yet.

## Examples
To run examples first check which are available:
```
cargo run --example
```

Then run the desired one with
```
cargo run --example <example_name>
```

## License
`devotee` is distributed under the MIT license.
