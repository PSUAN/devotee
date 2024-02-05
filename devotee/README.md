# devotee
A bit personal visualization engine.

`devotee` is directly based on:

- [winit](https://crates.io/crates/winit) - Rust windowing library;
- [rodio](https://crates.io/crates/rodio) - `cpal`-based audio playback library;

## Backends

`devotee` utilizes a backend to render data to.
Currently there are two backends:

- `back-softbuffer` - [softbuffer](https://crates.io/crates/softbuffer)-based backend.
- `back-pixels` - [pixels](https://crates.io/crates/pixels)-based backend.

## Goals

`devotee` aims to provide primitive pixel-perfect visualization and optional sound effects.

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
