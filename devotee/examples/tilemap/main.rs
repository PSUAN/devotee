use devotee::app;
use devotee::app::config;
use devotee::app::context::UpdateContext;
use devotee::app::input::VirtualKeyCode;
use devotee::app::setup;
use devotee::node::Node;
use devotee::util::vector::Vector;
use devotee::visual::canvas::Canvas;
use devotee::visual::color;
use devotee::visual::prelude::*;
use devotee::visual::sprite::Sprite;

use std::f64::consts::PI;

fn main() {
    let init_config = setup::Setup::<Config>::new(|_| TilemapNode::new())
        .with_title("tilemap")
        .with_resolution((128, 128))
        .with_scale(2);
    let app = app::App::with_setup(init_config).unwrap();

    app.run();
}

struct Config;

impl config::Config for Config {
    type Node = TilemapNode;
    type Palette = FourBits;
    type Converter = Converter;

    fn converter() -> Self::Converter {
        Converter { transparent: None }
    }

    fn background_color() -> Self::Palette {
        FourBits::Black
    }

    fn window_background_color() -> [u8; 3] {
        [0, 0, 0]
    }
}

struct TilemapNode {
    counter: f64,
    tile_data: [usize; 8 * 8],
    tiles: [Sprite<FourBits, 8, 8>; 8],
}

impl TilemapNode {
    fn new() -> Self {
        let counter = 0.0;
        let tile_data = [
            1, 1, 1, 1, 0, 0, 1, 2, 1, 0, 0, 1, 0, 3, 4, 5, 1, 0, 0, 0, 0, 6, 7, 0, 1, 1, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 2, 0, 0, 2, 0, 0,
            0, 0, 2, 2, 2, 2,
        ];
        let tiles = [0, 1, 2, 3, 4, 5, 6, 7].map(|i| Self::generate_tile((i * 2).into()));
        Self {
            counter,
            tile_data,
            tiles,
        }
    }

    fn generate_tile(color: FourBits) -> Sprite<FourBits, 8, 8> {
        let mut sprite = Sprite::with_color(FourBits::Black);
        if color != FourBits::Black {
            sprite.rect((0, 0), (8, 8), paint(color));
            sprite.filled_rect((2, 2), (6, 6), paint(color));
            sprite.line((1, 1), (6, 6), paint(FourBits::Beige));
            sprite.line((6, 1), (1, 6), paint(FourBits::White));
        }
        sprite
    }
}

impl<'a> Node<&mut UpdateContext<'a>, &mut Canvas<FourBits>> for TilemapNode {
    fn update(&mut self, update: &mut UpdateContext<'_>) {
        if update.input().just_key_pressed(VirtualKeyCode::Escape) {
            update.shutdown();
        }

        self.counter += update.delta().as_secs_f64();
    }

    fn render(&self, render: &mut Canvas<FourBits>) {
        render.clear(0.into());

        let mut tile_data = self.tile_data.into_iter();
        let tiles: &[Sprite<_, 8, 8>] = &self.tiles;
        render.rect((32, 32), (32 + 64, 32 + 64), paint(7.into()));
        render.tilemap(
            (32, 32),
            |i, _| {
                let x = 8 * (i as i32 % 8);
                let y = 8 * (i as i32 / 8);
                Vector::new(
                    x,
                    y + (2.0 * (self.counter * PI + x as f64 / 32.0 * PI).sin()).round() as i32,
                )
            },
            &tiles,
            &mut tile_data,
            |_, _, p, _, _, o| p.mix(o),
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

impl color::Color for FourBits {
    fn mix(self, other: Self) -> Self {
        match other {
            FourBits::Black => self,
            value => value,
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
