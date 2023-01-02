use devotee::app;
use devotee::app::config;
use devotee::app::context::UpdateContext;
use devotee::app::input::{Keyboard, VirtualKeyCode};
use devotee::app::setup;
use devotee::node::Node;
use devotee::visual::canvas::Canvas;
use devotee::visual::color;
use devotee::visual::prelude::*;
use devotee::visual::sprite::Sprite;

use std::collections::HashMap;
use std::f64::consts::{FRAC_PI_2, PI};

fn main() {
    let init_config = setup::Setup::<Config>::new(|_| TextNode::new(), Default::default())
        .with_title("text")
        .with_resolution((128, 128))
        .with_scale(2);
    let app = app::App::with_setup(init_config).unwrap();

    app.run();
}

struct Config;

impl config::Config for Config {
    type Node = TextNode;
    type Palette = FourBits;
    type Converter = Converter;
    type Input = Keyboard;

    fn converter() -> Self::Converter {
        Converter { transparent: None }
    }

    fn background_color() -> Self::Palette {
        FourBits::Black
    }
}

struct TextNode {
    counter: f64,
    moving: bool,
    ease: (f64, f64),
    font: HashMap<char, Sprite<u8, 4, 6>>,
    angle: f64,
}

impl TextNode {
    fn new() -> Self {
        let counter = 0.0;
        // FIXME: replace with array::zip of chars and symbols arrays when
        // stabilized.
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

impl Node<&mut UpdateContext<Keyboard>, &mut Canvas<FourBits>> for TextNode {
    fn update(&mut self, update: &mut UpdateContext<Keyboard>) {
        if update.input().just_key_pressed(VirtualKeyCode::Escape) {
            update.shutdown();
        }
        if update.input().just_key_pressed(VirtualKeyCode::Z) && !self.moving {
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

    fn render(&self, render: &mut Canvas<FourBits>) {
        let color = FourBits::White;

        render.clear(0.into());
        render.line((64 - 16, 64), (64 + 16, 64), paint(FourBits::Red));
        render.line((64, 64 - 16), (64, 64 + 16), paint(FourBits::Green));
        render.text((64, 64), printer(), &self.font, "0", |_, _, p, _, _, o| {
            if o == 0 {
                p
            } else {
                color
            }
        });

        let cos = self.angle.cos();
        let sin = self.angle.sin();

        let x = 64 + (32.0 * cos) as i32;
        let y = 64 - (32.0 * sin) as i32;

        render.line((64, 64), (x, y), paint(FourBits::Beige));

        render.text(
            (x, y),
            printer(),
            &self.font,
            &format!("{:.3}\n{:.3}", cos, sin),
            |_, _, p, _, _, o| {
                if o == 0 {
                    p
                } else {
                    color
                }
            },
        );
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
