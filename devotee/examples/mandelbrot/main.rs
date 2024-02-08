use std::collections::HashSet;

use devotee::app;
use devotee::app::context::Context;
use devotee::app::input::event;
use devotee::app::input::Input;
use devotee::app::root::Root;
use devotee::app::setup;
use devotee::app::window::Window;
use devotee::util::vector::Vector;
use devotee::visual::canvas::Canvas;
use devotee::visual::prelude::*;
use devotee::winit;
use devotee_backend_softbuffer::SoftbufferBackend;

fn main() {
    let init_config = setup::Builder::new()
        .with_render_target(Canvas::with_resolution(FourBits::Black, 128, 128))
        .with_input(Default::default())
        .with_root_constructor(|_| Default::default())
        .with_title("mandelbrot")
        .with_scale(4);
    let app = app::App::<Mandelbrot, SoftbufferBackend>::with_setup(init_config).unwrap();

    app.run();
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

impl Root for Mandelbrot {
    type Converter = Converter;
    type Input = CustomInput;
    type RenderTarget = Canvas<FourBits>;

    fn update(&mut self, update: &mut Context<CustomInput>) {
        let delta = update.delta().as_secs_f64();

        if update.input().is_pressed(Button::In) {
            self.scale -= delta;
        }
        if update.input().is_pressed(Button::Out) {
            self.scale += delta;
        }

        let scale = 2.0_f64.powf(self.scale);
        if update.input().is_pressed(Button::Left) {
            *self.center.x_mut() += delta / scale;
        }
        if update.input().is_pressed(Button::Right) {
            *self.center.x_mut() -= delta / scale;
        }
        if update.input().is_pressed(Button::Up) {
            *self.center.y_mut() += delta / scale;
        }
        if update.input().is_pressed(Button::Down) {
            *self.center.y_mut() -= delta / scale;
        }

        if update.input().just_pressed(Button::Quit) {
            update.shutdown();
        }
    }

    fn render(&self, render: &mut Canvas<FourBits>) {
        let scale = 2.0_f64.powf(self.scale);
        let width = render.width();
        let height = render.height();
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
                if let Some(p) = render.pixel_mut((x, y).into()) {
                    *p = (iteration % 16).into();
                }
            }
        }
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

impl devotee_backend::Converter for Converter {
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

#[derive(Default)]
struct CustomInput {
    is_pressed: HashSet<Button>,
    was_pressed: HashSet<Button>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum Button {
    Quit,
    Left,
    Right,
    Up,
    Down,
    In,
    Out,
}

impl TryFrom<Option<event::VirtualKeyCode>> for Button {
    type Error = ();

    fn try_from(value: Option<event::VirtualKeyCode>) -> Result<Self, Self::Error> {
        match value.ok_or(())? {
            event::VirtualKeyCode::Escape => Ok(Button::Quit),
            event::VirtualKeyCode::Left => Ok(Button::Left),
            event::VirtualKeyCode::Right => Ok(Button::Right),
            event::VirtualKeyCode::Up => Ok(Button::Up),
            event::VirtualKeyCode::Down => Ok(Button::Down),
            event::VirtualKeyCode::Z => Ok(Button::In),
            event::VirtualKeyCode::X => Ok(Button::Out),
            _ => Err(()),
        }
    }
}

impl CustomInput {
    pub fn is_pressed(&self, button: Button) -> bool {
        self.is_pressed.contains(&button)
    }

    pub fn just_pressed(&self, button: Button) -> bool {
        self.is_pressed.contains(&button) && !self.was_pressed.contains(&button)
    }
}

impl<Bck> Input<Bck> for CustomInput {
    fn next_frame(&mut self) {
        self.was_pressed = self.is_pressed.clone();
    }

    fn consume_window_event<'a>(
        &mut self,
        event: winit::event::WindowEvent<'a>,
        _window: &Window,
        _back: &Bck,
    ) -> Option<winit::event::WindowEvent<'a>> {
        match event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => {
                if let Ok(button) = input.virtual_keycode.try_into() {
                    match input.state {
                        event::ElementState::Pressed => {
                            self.is_pressed.insert(button);
                        }
                        event::ElementState::Released => {
                            self.is_pressed.remove(&button);
                        }
                    }
                }
                None
            }
            event => Some(event),
        }
    }
}
