use devotee::app;
use devotee::app::config;
use devotee::app::context::UpdateContext;
use devotee::app::input::VirtualKeyCode;
use devotee::app::setup;
use devotee::math::vector::Vector;
use devotee::node::Node;
use devotee::visual::canvas::Canvas;
use devotee::visual::color;

fn main() {
    let init_config = setup::Setup::<Config>::default()
        .with_title("mandelbrot")
        .with_resolution((128, 128))
        .with_scale(4);
    let app = app::App::with_config(init_config).unwrap();

    app.run();
}

struct Config;

impl config::Config for Config {
    type Node = Mandelbrot;
    type Palette = FourBits;
    type Converter = Converter;

    fn converter() -> Self::Converter {
        Converter { transparent: None }
    }

    fn background_color() -> Self::Palette {
        0.into()
    }

    fn window_background_color() -> [u8; 3] {
        [0, 0, 0]
    }
}

struct Mandelbrot {
    scale: f64,
    center: Vector<f64>,
}

impl Default for Mandelbrot {
    fn default() -> Self {
        Self {
            scale: 0.5,
            center: Vector::new(0.0, 0.0),
        }
    }
}

impl<'a> Node<&mut UpdateContext<'a>, &mut Canvas<FourBits>> for Mandelbrot {
    fn update(&mut self, update: &mut UpdateContext<'_>) {
        let delta = update.delta().as_secs_f64();

        if update.input().is_key_pressed(VirtualKeyCode::Z)
            || update.input().is_key_pressed(VirtualKeyCode::C)
        {
            self.scale -= delta;
        }
        if update.input().is_key_pressed(VirtualKeyCode::X) {
            self.scale += delta;
        }

        let scale = 2.0_f64.powf(self.scale);
        if update.input().is_key_pressed(VirtualKeyCode::Left) {
            *self.center.x_mut() += delta / scale;
        }
        if update.input().is_key_pressed(VirtualKeyCode::Right) {
            *self.center.x_mut() -= delta / scale;
        }
        if update.input().is_key_pressed(VirtualKeyCode::Up) {
            *self.center.y_mut() += delta / scale;
        }
        if update.input().is_key_pressed(VirtualKeyCode::Down) {
            *self.center.y_mut() -= delta / scale;
        }

        if update.input().just_key_pressed(VirtualKeyCode::Escape) {
            update.shutdown();
        }
        if update.input().just_key_pressed(VirtualKeyCode::Return)
            && update.input().is_key_pressed(VirtualKeyCode::LAlt)
        {
            update.add_window_command(|window| window.set_fullscreen(!window.is_fullscreen()));
        }
    }

    fn render(&self, render: &mut Canvas<FourBits>) {
        let scale = 2.0_f64.powf(self.scale);
        let width = render.width() as i32;
        let height = render.height() as i32;
        for x in 0..width {
            for y in 0..height {
                let x0 = (x - width / 2) as f64 / width as f64 / scale - self.center.x();
                let y0 = (y - height / 2) as f64 / height as f64 / scale - self.center.y();
                let mut px = 0.0;
                let mut py = 0.0;
                let mut iteration: u8 = 0;
                while px * px + py * py < 4.0 && iteration < 32 {
                    (px, py) = (px * px - py * py + x0, 2.0 * px * py + y0);
                    iteration += 1;
                }
                render.draw_pixel((x, y), (iteration / 2).into());
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum FourBits {
    Black,
    DarkBlue,
    Eggplant,
    DarkGreen,
    Brown,
    DirtyGray,
    Gray,
    White,
    Red,
    Orange,
    Yellow,
    Green,
    LightBlue,
    Purple,
    Pink,
    Beige,
}

impl From<u8> for FourBits {
    #[inline]
    fn from(value: u8) -> Self {
        match value {
            0 => FourBits::Black,
            1 => FourBits::DarkBlue,
            2 => FourBits::Eggplant,
            3 => FourBits::DarkGreen,
            4 => FourBits::Brown,
            5 => FourBits::DirtyGray,
            6 => FourBits::Gray,
            7 => FourBits::White,
            8 => FourBits::Red,
            9 => FourBits::Orange,
            10 => FourBits::Yellow,
            11 => FourBits::Green,
            12 => FourBits::LightBlue,
            13 => FourBits::Purple,
            14 => FourBits::Pink,
            15 => FourBits::Beige,
            _ => FourBits::Black,
        }
    }
}

impl color::Color for FourBits {
    #[inline]
    fn mix(self, other: Self) -> Self {
        other
    }
}

struct Converter {
    transparent: Option<FourBits>,
}

impl color::Converter for Converter {
    type Palette = FourBits;
    #[inline]
    fn convert(&self, color: &Self::Palette) -> [u8; 4] {
        if matches!(&self.transparent, Some(transparent) if *transparent == *color) {
            return [0x00, 0x00, 0x00, 0x00];
        }
        {
            match color {
                FourBits::Black => [0x00, 0x00, 0x00, 0xff],
                FourBits::DarkBlue => [0x1d, 0x2b, 0x53, 0xff],
                FourBits::Eggplant => [0x7e, 0x25, 0x53, 0xff],
                FourBits::DarkGreen => [0x00, 0x87, 0x51, 0xff],
                FourBits::Brown => [0xab, 0x52, 0x36, 0xff],
                FourBits::DirtyGray => [0x5f, 0x57, 0x4f, 0xff],
                FourBits::Gray => [0xc2, 0xc3, 0xc7, 0xff],
                FourBits::White => [0xff, 0xf1, 0xe8, 0xff],
                FourBits::Red => [0xff, 0x00, 0x4d, 0xff],
                FourBits::Orange => [0xff, 0xa3, 0x00, 0xff],
                FourBits::Yellow => [0xff, 0xec, 0x27, 0xff],
                FourBits::Green => [0x00, 0xe4, 0x36, 0xff],
                FourBits::LightBlue => [0x29, 0xad, 0xff, 0xff],
                FourBits::Purple => [0x83, 0x76, 0x9c, 0xff],
                FourBits::Pink => [0xff, 0x77, 0xa8, 0xff],
                FourBits::Beige => [0xff, 0xcc, 0xaa, 0xff],
            }
        }
    }
}
