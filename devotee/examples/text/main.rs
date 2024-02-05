use std::collections::HashMap;
use std::f64::consts::{FRAC_PI_2, PI};

use devotee::app;
use devotee::app::context::Context;
use devotee::app::input::key_mouse::{KeyMouse, VirtualKeyCode};
use devotee::app::root::Root;
use devotee::app::setup;
use devotee::util::vector::Vector;
use devotee::visual::color;
use devotee::visual::prelude::*;
use devotee::visual::sprite::Sprite;
use devotee_backend_softbuffer::SoftbufferBackend;

fn main() {
    let init_config = setup::Builder::new()
        .with_render_target(Sprite::with_color(FourBits::Black))
        .with_input(Default::default())
        .with_root_constructor(|_| TextApp::new())
        .with_title("text")
        .with_scale(2);
    let app = app::App::<TextApp, SoftbufferBackend>::with_setup(init_config).unwrap();

    app.run();
}

struct TextApp {
    counter: f64,
    moving: bool,
    ease: (f64, f64),
    font: HashMap<char, Sprite<u8, 4, 6>>,
    angle: f64,
}

impl TextApp {
    fn new() -> Self {
        let counter = 0.0;
        let font = HashMap::from([
            (
                '0',
                symbol([[1, 1, 1], [1, 0, 1], [1, 0, 1], [1, 0, 1], [1, 1, 1]]),
            ),
            (
                '1',
                symbol([[1, 1, 0], [0, 1, 0], [0, 1, 0], [0, 1, 0], [1, 1, 1]]),
            ),
            (
                '2',
                symbol([[1, 1, 1], [0, 0, 1], [1, 1, 1], [1, 0, 0], [1, 1, 1]]),
            ),
            (
                '3',
                symbol([[1, 1, 1], [0, 0, 1], [1, 1, 1], [0, 0, 1], [1, 1, 1]]),
            ),
            (
                '4',
                symbol([[1, 0, 1], [1, 0, 1], [1, 1, 1], [0, 0, 1], [0, 0, 1]]),
            ),
            (
                '5',
                symbol([[1, 1, 1], [1, 0, 0], [1, 1, 1], [0, 0, 1], [1, 1, 1]]),
            ),
            (
                '6',
                symbol([[1, 1, 1], [1, 0, 0], [1, 1, 1], [1, 0, 1], [1, 1, 1]]),
            ),
            (
                '7',
                symbol([[1, 1, 1], [0, 0, 1], [0, 0, 1], [0, 0, 1], [0, 0, 1]]),
            ),
            (
                '8',
                symbol([[1, 1, 1], [1, 0, 1], [1, 1, 1], [1, 0, 1], [1, 1, 1]]),
            ),
            (
                '9',
                symbol([[1, 1, 1], [1, 0, 1], [1, 1, 1], [0, 0, 1], [0, 0, 1]]),
            ),
            (
                '.',
                symbol([[0, 0, 0], [0, 0, 0], [0, 0, 0], [0, 0, 0], [0, 1, 0]]),
            ),
            (
                '-',
                symbol([[0, 0, 0], [0, 0, 0], [1, 1, 1], [0, 0, 0], [0, 0, 0]]),
            ),
            (
                '\n',
                symbol([[0, 0, 0], [0, 0, 0], [0, 0, 0], [0, 0, 0], [0, 0, 0]]),
            ),
        ]);
        let moving = false;
        let ease = (0.0, FRAC_PI_2);
        let angle = 0.0;
        Self {
            counter,
            moving,
            font,
            ease,
            angle,
        }
    }
}

fn symbol(data: [[u8; 3]; 5]) -> Sprite<u8, 4, 6> {
    let data = data.map(|line| [line[0], line[1], line[2], 0]);
    let data = [data[0], data[1], data[2], data[3], data[4], [0, 0, 0, 0]];
    Sprite::with_data(data)
}

impl Root for TextApp {
    type Converter = Converter;
    type Input = KeyMouse;
    type RenderTarget = Sprite<FourBits, 128, 128>;

    fn update(&mut self, update: &mut Context<KeyMouse>) {
        if update.input().keys().just_pressed(VirtualKeyCode::Escape) {
            update.shutdown();
        }
        if update.input().keys().just_pressed(VirtualKeyCode::Z) && !self.moving {
            self.moving = true;
            self.counter = 0.0;
        }

        if self.moving {
            self.counter += update.delta().as_secs_f64();

            if self.counter >= 1.0 {
                self.moving = false;
                self.angle = self.ease.1;
                self.ease.0 = self.ease.1;
                self.ease.1 += FRAC_PI_2;
            } else {
                let t = -((PI * self.counter).cos() - 1.0) / 2.0;
                self.angle = self.ease.0 + (self.ease.1 - self.ease.0) * t;
            }
        }
    }

    fn render(&self, render: &mut Sprite<FourBits, 128, 128>) {
        let color = FourBits::White;

        let mut render = render.painter();
        render.set_offset(Vector::new(64, 64));

        render.clear(0.into());
        render.line((-16, 0), (16, 0), paint(FourBits::Red));
        render.line((0, -16), (0, 16), paint(FourBits::Green));

        let text_painter = |_, _, p, _, _, o| {
            if o == 0 {
                p
            } else {
                color
            }
        };

        let cos = self.angle.cos();
        let sin = self.angle.sin();

        let x = (32.0 * cos) as i32;
        let y = -(32.0 * sin) as i32;

        render.line((0, 0), (x, y), paint(FourBits::Beige));

        render.text((0, 0), printer(), &self.font, "0", text_painter);
        render.text(
            (x, y),
            printer(),
            &self.font,
            &format!("{:.3}\n{:.3}", cos, sin),
            text_painter,
        );
    }

    fn converter(&self) -> &Converter {
        &Converter
    }

    fn background_color(&self) -> FourBits {
        FourBits::Black
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

struct Converter;

impl color::Converter for Converter {
    type Palette = FourBits;
    #[inline]
    fn convert(&self, color: &Self::Palette) -> u32 {
        match color {
            FourBits::Black => 0x00000000,
            FourBits::DarkBlue => 0x001d2b53,
            FourBits::Eggplant => 0x007e2553,
            FourBits::DarkGreen => 0x00008751,
            FourBits::Brown => 0x00ab5236,
            FourBits::DirtyGray => 0x005f574f,
            FourBits::Gray => 0x00c2c3c7,
            FourBits::White => 0x00fff1e8,
            FourBits::Red => 0x00ff004d,
            FourBits::Orange => 0x00ffa300,
            FourBits::Yellow => 0x00ffec27,
            FourBits::Green => 0x0000e436,
            FourBits::LightBlue => 0x0029adff,
            FourBits::Purple => 0x0083769c,
            FourBits::Pink => 0x00ff77a8,
            FourBits::Beige => 0x00ffccaa,
        }
    }
}
