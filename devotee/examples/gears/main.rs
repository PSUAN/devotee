use std::f32::consts::{self, PI};
use std::time::Duration;

use devotee::app::root::Root;
use devotee::app::App;
use devotee::input::winit_input::{KeyCode, Keyboard};
use devotee::util::vector::Vector;
use devotee::visual::canvas::Canvas;
use devotee::visual::{paint, Paint, PaintTarget, Painter};
use devotee_backend::{Context, Converter};
use devotee_backend_softbuffer::{Error, SoftBackend, SoftContext, SoftInit, SoftMiddleware};
use winit::window::Fullscreen;

fn main() -> Result<(), Error> {
    let backend = SoftBackend::try_new("gears")?;
    backend.run(
        App::new(Gears::new()),
        SoftMiddleware::new(Canvas::with_resolution(false, 320, 240), Keyboard::new())
            .with_background_color(0xff000000),
        Duration::from_secs_f32(1.0 / 60.0),
    )
}

struct Gears {
    drive_gear: Gear,
    driven_gear: Gear,
}

impl Gears {
    fn new() -> Self {
        let mut drive_gear = Gear::new_gear(128.0, 20);
        drive_gear.center = Vector::new(0.0, 240.0);
        let mut driven_gear = Gear::new_gear(384.0, 60);
        driven_gear.center = Vector::new(256.0, 240.0);
        Self {
            drive_gear,
            driven_gear,
        }
    }
}

impl Root<SoftInit<'_>, SoftContext<'_, Keyboard>> for Gears {
    type Converter = TwoConverter;
    type RenderSurface = Canvas<bool>;

    fn init(&mut self, init: &mut SoftInit) {
        init.control()
            .window_ref()
            .set_fullscreen(Some(Fullscreen::Borderless(None)));

        self.driven_gear.angle =
            -self.drive_gear.angle / 3.0 + PI / self.driven_gear.teeth_count as f32;
    }

    fn update(&mut self, context: &mut SoftContext<Keyboard>) {
        let keyboard = context.input();

        if keyboard.is_pressed(KeyCode::Space) {
            self.drive_gear.angle += PI * context.delta().as_secs_f32() / 4.0;
        }
        self.driven_gear.angle =
            -self.drive_gear.angle / 3.0 + PI / self.driven_gear.teeth_count as f32;

        if keyboard.just_pressed(KeyCode::Escape) {
            context.shutdown();
        }
    }

    fn render(&self, render: &mut Self::RenderSurface) {
        let mut render = render.painter();
        render.clear(false);
        self.drive_gear.render(&mut render);
        self.driven_gear.render(&mut render);
    }

    fn converter(&self) -> Self::Converter {
        TwoConverter
    }
}

struct TwoConverter;

impl Converter for TwoConverter {
    type Data = bool;

    fn convert(&self, _: usize, _: usize, data: Self::Data) -> u32 {
        if data {
            0xffc0b0a0
        } else {
            0xff101020
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

    fn render(&self, painter: &mut Painter<Canvas<bool>, f32>) {
        painter.set_offset(self.center);
        for i in 0..=self.teeth_count {
            let angle = i as f32 * 2.0 * PI / self.teeth_count as f32 + self.angle;
            let tooth = self
                .tooth
                .iter()
                .copied()
                .map(|v| rotate_vector(v, angle))
                .collect::<Vec<_>>();
            painter.polygon_f(&tooth, paint(true));
        }
        painter.circle_f((0.0, 0.0), self.internal_radius, paint(true));
        painter.circle_b((0.0, 0.0), self.separation_diameter / 2.0, |_, _, p| !p);
        for i in 0..3 {
            let angle = PI * 2.0 * i as f32 / 3.0;
            painter.circle_f(
                rotate_vector((self.internal_radius / 2.0, 0.0).into(), self.angle + angle),
                self.internal_radius / 4.0,
                |x, y, _| (x + y) % 2 == 0,
            );
        }
        painter.circle_f((0.0, 0.0), self.internal_radius / 2.0, paint(false));
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
