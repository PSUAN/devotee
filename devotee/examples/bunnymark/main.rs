use devotee::app;
use devotee::app::config;
use devotee::app::context::UpdateContext;
use devotee::app::input::VirtualKeyCode;
use devotee::app::setup;
use devotee::math::vector::Vector;
use devotee::node::Node;
use devotee::visual::canvas::Canvas;
use devotee::visual::color;
use std::time::{Duration, Instant};

const BUNNY_WIDTH: usize = 8;
const BUNNY_HEIGHT: usize = 16;
const WIDTH: usize = 128;
const HEIGHT: usize = 128;
const ACCELERATION: f64 = 8.0;

fn main() {
    let init_config = setup::Setup::<Config>::default()
        .with_title("bunnymark")
        .with_resolution((WIDTH, HEIGHT))
        .with_update_delay(Duration::from_secs_f64(1.0 / 60.0));
    let app = app::App::with_setup(init_config).unwrap();

    app.run();
}

struct Config;

impl config::Config for Config {
    type Node = BunnyMark;
    type Palette = FourBits;
    type Converter = Converter;

    fn converter() -> Self::Converter {
        Converter
    }

    fn background_color() -> Self::Palette {
        FourBits::Black
    }

    fn window_background_color() -> [u8; 3] {
        [0, 0, 0]
    }
}

pub struct Converter;

impl color::Converter for Converter {
    type Palette = FourBits;
    #[inline]
    fn convert(&self, color: &Self::Palette) -> [u8; 4] {
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

struct BunnyMark {
    bunnies: Vec<Bunny>,
    texture: Canvas<<Config as config::Config>::Palette>,
    counter: i32,
    previous: Instant,
}

impl Default for BunnyMark {
    fn default() -> Self {
        let bunnies = Vec::new();
        let mut texture = Canvas::with_resolution(0.into(), BUNNY_WIDTH, BUNNY_HEIGHT);
        texture.draw_filled_rect((1, 0), (2, 15), FourBits::White);
        texture.draw_filled_rect((5, 0), (6, 15), FourBits::White);
        texture.draw_filled_rect((0, 5), (8, 10), FourBits::White);
        texture.draw_filled_rect((3, 11), (4, 14), FourBits::White);
        texture.draw_pixel((2, 7), FourBits::Pink);
        texture.draw_pixel((5, 7), FourBits::Pink);
        texture.draw_line((7, 4), (7, 8), FourBits::Gray);
        texture.draw_line((6, 9), (6, 15), FourBits::Gray);
        let counter = 0;
        let previous = Instant::now();
        Self {
            bunnies,
            texture,
            counter,
            previous,
        }
    }
}

impl BunnyMark {
    fn add_bunny(&mut self) {
        for i in 0..32 {
            self.bunnies.push(Bunny::new(i as f64));
        }
    }
}

impl<'a> Node<&mut UpdateContext<'a>, &mut Canvas<FourBits>> for BunnyMark {
    fn update(&mut self, update: &mut UpdateContext<'_>) {
        if update.input().is_key_pressed(VirtualKeyCode::Space) {
            self.add_bunny();
        }

        let delta = update.delta().as_secs_f64();
        self.counter += 1;

        let now = Instant::now();
        let real_delta = now - self.previous;
        if real_delta > Duration::from_secs_f64(0.25) {
            let real_delta = real_delta.as_secs_f64();
            println!("Bunny count: {}", self.bunnies.len());
            println!(
                "{} updates in {} seconds makes {} FPS",
                self.counter,
                real_delta,
                self.counter as f64 / real_delta
            );
            self.previous = now;
            self.counter = 0;
        }

        for bunny in self.bunnies.iter_mut() {
            Bunny::update(bunny, delta);
        }
    }

    fn render(&self, render: &mut Canvas<FourBits>) {
        render.clear(FourBits::Black);
        for bunny in self.bunnies.iter() {
            render.draw_image(
                (bunny.pose.x() as i32, bunny.pose.y() as i32),
                &self.texture,
            );
        }
    }
}

struct Bunny {
    pose: Vector<f64>,
    velocity: Vector<f64>,
}

impl Bunny {
    fn new(offset_vel: f64) -> Self {
        let pose = (1.0, 1.0).into();
        let velocity = (8.0 + offset_vel, offset_vel).into();
        Self { pose, velocity }
    }

    fn update(&mut self, delta: f64) {
        *self.velocity.y_mut() += ACCELERATION;
        self.pose = self.pose + self.velocity * delta;
        if self.pose.x() < 0.0 {
            *self.velocity.x_mut() = self.velocity.x().abs();
        }
        if self.pose.y() < 0.0 {
            *self.velocity.y_mut() = self.velocity.y().abs();
        }
        if self.pose.x() > (WIDTH - BUNNY_WIDTH) as f64 {
            *self.velocity.x_mut() = -self.velocity.x().abs();
        }
        if self.pose.y() > (HEIGHT - BUNNY_HEIGHT) as f64 {
            *self.pose.y_mut() = (HEIGHT - BUNNY_HEIGHT) as f64;
            *self.velocity.y_mut() = -self.velocity.y().abs()
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum FourBits {
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
