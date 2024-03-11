# devotee

Simplistic visualization project.

## Using devotee

### Creating an app

Devotee app is represented by the `Root` implementation and a `Backend` system.

#### Root implementation

`Root` implementor must choose desired input system, pixel data converter, and render surface.
Also, it has to implement `update`, `render` and `converter` methods.

Minimalist `Root` implementation may look like this:

```rust
struct Minimal;

impl Root for Minimal {
    type Input = NoInput;
    type Converter = BlackWhiteConverter;
    type RenderSurface = Canvas<bool>;

    fn update(&mut self, _: AppContext<Self::Input>) {}

    fn render(&self, _: &mut Self::RenderSurface) {}

    fn converter(&self) -> Self::Converter {
        BlackWhiteConverter
    }
}
```

This `Root` implementation:

- Uses `NoInput` as input system;
- Relies on the `BlackWhiteConverter` (implementation will be discussed later) to convert data of the `RenderSurface`;
- Uses `Canvas` with `bool` pixels as a `RenderSurface`;
- Does nothing during `update`;
- Draws nothing during `render`;
- Returns `BlackWhiteConverter` instance for data conversion;

The sample `BlackWhiteConverter` is implemented as:

```rust
struct BlackWhiteConverter;

impl Converter for BlackWhiteConverter {
    type Data = bool;

    fn convert(&self, _x: usize, _y: usize, data: Self::Data) -> u32 {
        if data {0xffffffff} else {0xff000000}
    }
}
```

It ignores `x` and `y` coordinates of the pixel and returns either pure white or pure black depending on the `data` value.

#### Backend usage

So, with the `Root` being implemented it is time to launch it using some backend.

For this example we will rely on the [Softbuffer](https://crates.io/crates/softbuffer)-based backend [implementation](https://crates.io/crates/devotee-backend-softbuffer).

```rust
fn main() -> Result<(), Error> {
    let backend = SoftBackend::try_new("minimal")?;
    backend.run(
        App::new(Minimal),
        SoftMiddleware::new(Canvas::with_resolution(false, 128, 128), NoInput),
        Duration::from_secs_f32(1.0 / 60.0),
    )
}
```

### Updating app state

Consider `Extended` implementation of `Root`.

```rust
struct Extended {
    counter: f32,
}
```

Let it use `Keyboard` as input.
It shuts down on the `Escape` button being pressed.
Also, it counts passed simulation time in `counter`.

So, first part of its implementation looks like this:

```rust
impl Root for Extended {
    type Input = Keyboard;
    type Converter = BlackWhiteConverter;
    type RenderSurface = Canvas<bool>;

    fn update(&mut self, mut context: devotee::app::AppContext<Self::Input>) {
        if context.input().just_pressed(KeyCode::Escape) {
            context.shutdown();
        }

        self.counter += context.delta().as_secs_f32();
    }

    // ...
}
```

During render it cleans render surface, calculates the surface center and draws two filled circles using `painter`.
`Painter` instance accepts functions as arguments instead of pure colors.
The function decides what to do with the pixel passed given its coordinates.
`paint` is a predefined function to override any original value.

Note that there are two implementations of `painter`: for `i32` coordinates and (subpixel one) for `f32` coordinates.

```rust
    //. ..
    fn render(&self, surface: &mut Self::RenderSurface) {
        surface.clear(false);
        let center = surface.dimensions().map(|a| a as f32) / 2.0;

        let mut painter = surface.painter();
        let radius = 48.0 + 16.0 * self.counter.sin();

        painter.circle_f(center, radius, paint(true));
        painter.circle_f(center, radius / 2.0, |x, y, _| (x + y) % 2 == 0)
    }
    // ...
```

## Examples

There are some examples in the `examples` folder.

## License

`devotee` is licensed under the `MIT` license.
