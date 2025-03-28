use std::f32::consts::{self};
use std::ops::{Deref, DerefMut};

use devotee::app::root::Root;
use devotee::app::App;
use devotee::input::winit_input::{KeyCode, Keyboard};
use devotee::util::vector::Vector;

use devotee::visual::adapter::generic::Adapter;
use devotee::visual::adapter::Converter;
use devotee::visual::image::{DesignatorMut, DesignatorRef, ImageMut};
use devotee::visual::{paint, Paint, Painter};
use devotee_backend::middling::MiddlingMiddleware;
use devotee_backend_softbuffer::{Error, SoftBackend, SoftContext, SoftInit, SoftSurface};

fn main() -> Result<(), Error> {
    let gears = Gears::new();
    let app = App::new(gears);
    let middleware = MiddlingMiddleware::new(app, Keyboard::new());
    let mut backend = SoftBackend::new(middleware);

    backend.run()
}

struct Gears {
    drive_gear: Gear,
    driven_gear: Gear,
}

impl Gears {
    fn new() -> Self {
        let mut drive_gear = Gear::new_gear(128.0, 20);
        drive_gear.center = Vector::new(0.0, 32.0);
        let mut driven_gear = Gear::new_gear(384.0, 60);
        driven_gear.center = Vector::new(256.0, 32.0);
        Self {
            drive_gear,
            driven_gear,
        }
    }
}

impl Root<SoftInit<'_>, SoftContext<'_>, Keyboard, SoftSurface<'_>> for Gears {
    fn init(&mut self, init: &mut SoftInit) {
        init.set_render_window_size(320, 240);
        init.set_title("Gears demo: press ESC to exit.");

        self.driven_gear.angle =
            -self.drive_gear.angle / 3.0 + consts::PI / self.driven_gear.teeth_count as f32;
    }

    fn update(&mut self, context: &mut SoftContext, input: &Keyboard) {
        let keyboard = input;

        if keyboard.is_pressed(KeyCode::Space) {
            self.drive_gear.angle += consts::PI * context.delta().as_secs_f32() / 4.0;
        }
        self.driven_gear.angle =
            -self.drive_gear.angle / 3.0 + consts::PI / self.driven_gear.teeth_count as f32;

        if keyboard.just_pressed(KeyCode::Escape) {
            context.shutdown();
        }
    }

    fn render(&mut self, surface: &mut SoftSurface<'_>) {
        let mut adapter = Adapter::new(surface, &TwoConverter);
        adapter.clear(false);

        self.drive_gear.render(&mut adapter);
        self.driven_gear.render(&mut adapter);
    }
}

struct TwoConverter;

impl Converter for TwoConverter {
    type Pixel = bool;
    type Texel = u32;

    fn forward(&self, pixel: &Self::Pixel) -> Self::Texel {
        if *pixel {
            0xffc0b0a0
        } else {
            0xff101020
        }
    }

    fn inverse(&self, texel: &Self::Texel) -> Self::Pixel {
        match *texel & 0xffffff {
            0xc0b0a0 => true,
            _ => false,
        }
    }
}

const GEAR_PRECISION: usize = 16;

#[derive(Clone, Default, Debug)]
struct Gear {
    center: Vector<f32>,
    angle: f32,

    separation_diameter: f32,
    internal_radius: f32,
    teeth_count: usize,

    tooth: Vec<Vector<f32>>,
}

impl Gear {
    fn new_gear(separation_diameter: f32, teeth_count: usize) -> Self {
        let center = Vector::default();
        let angle = 0.0;
        let (internal_radius, tooth) = {
            let z = teeth_count as f32;
            let module = separation_diameter / teeth_count as f32;

            let h_a = module;
            let h_f = module * 1.25;

            let r = separation_diameter / 2.0 - h_f;
            let r_max = r + h_f + h_a;
            let r_mid = separation_diameter / 2.0;

            let a_max = (r_max.powi(2) - r.powi(2)).sqrt() / r;
            let a_mid = (r_mid.powi(2) - r.powi(2)).sqrt() / r;

            let x = |a: f32| r * a.cos() + r * a * a.sin();
            let y = |a: f32| r * a.sin() - r * a * a.cos();

            let ang_mid = f32::atan2(y(a_mid), x(a_mid));
            let ang_extra = consts::PI / z / 2.0 + ang_mid;

            let mut tooth = Vec::new();

            for i in 0..=GEAR_PRECISION {
                let a = (i as f32) / (GEAR_PRECISION as f32) * a_max;
                tooth.push(rotate_vector((x(a), y(a)).into(), -ang_extra));
            }
            for i in (0..=GEAR_PRECISION).rev() {
                let a = i as f32 / GEAR_PRECISION as f32 * a_max;
                tooth.push(rotate_vector((x(a), -y(a)).into(), ang_extra));
            }

            (r, tooth)
        };
        Self {
            center,
            angle,
            separation_diameter,
            internal_radius,
            teeth_count,
            tooth,
        }
    }

    fn render<I>(&self, image: &mut I)
    where
        I: ImageMut<Pixel = bool>,
        for<'a> <I as DesignatorRef<'a>>::PixelRef: Deref<Target = bool>,
        for<'a> <I as DesignatorMut<'a>>::PixelMut: DerefMut<Target = bool>,
    {
        let mut painter = Painter::new(image);
        painter.set_offset(self.center);
        for i in 0..=self.teeth_count {
            let angle = i as f32 * 2.0 * consts::PI / self.teeth_count as f32 + self.angle;
            let tooth = self
                .tooth
                .iter()
                .copied()
                .map(|v| rotate_vector(v, angle))
                .collect::<Vec<_>>();
            painter.polygon_f(&tooth, paint(true));
        }
        painter.circle_f(Vector::<f32>::zero(), self.internal_radius, paint(true));
        painter.circle_b(
            Vector::<f32>::zero(),
            self.separation_diameter / 2.0,
            |_, _, p| !p,
        );
        for i in 0..3 {
            let angle = consts::PI * 2.0 * i as f32 / 3.0;
            painter.circle_f(
                rotate_vector((self.internal_radius / 2.0, 0.0).into(), self.angle + angle),
                self.internal_radius / 4.0,
                |x, y, _| (x + y) % 2 == 0,
            );
        }
        painter.circle_f(
            Vector::<f32>::zero(),
            self.internal_radius / 2.0,
            paint(false),
        );
    }
}

fn rotate_vector(vector: Vector<f32>, rotation: f32) -> Vector<f32> {
    let cos = rotation.cos();
    let sin = rotation.sin();
    Vector::new(
        vector.x() * cos - vector.y() * sin,
        vector.y() * cos + vector.x() * sin,
    )
}
