use nannou::prelude::*;
// use ndarray::prelude::*;
// use ndarray::{Array, Ix2};
use rand::prelude::*;
use rand_pcg::Pcg64;

const SEED: u64 = 12345;
const WINDOW_X: u32 = 1000;
const WINDOW_Y: u32 = 1000;
const STELLAR_HALF_LIFE: u32 = 100000;
const N_INITIAL_STARS: u32 = 100;

#[derive(Copy, Clone)]
struct Star {
    w: f32,
    h: f32,
    x: f32,
    y: f32,
    age: u32,
    is_dead: bool
}

struct Model {
    _window: window::Id,
    rng: rand_pcg::Pcg64,
    stars: Vec<Star>
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(WINDOW_X, WINDOW_Y)
        .view(view)
        .build()
        .unwrap();
    let mut rng = Pcg64::seed_from_u64(SEED);
    let mut stars: Vec<Star> = Vec::new();
    for _ in 1..=N_INITIAL_STARS {
        let star: Star = create_star(app, &mut rng);
        stars.push(star);
    }

    Model {
        _window,
        rng,
        stars
    }
}

fn create_star(_app: &App, rng: &mut Pcg64) -> Star {
    let wh = _app.window_rect().wh();
    Star{
        w: rng.gen_range(1.0..3.0),
        h: rng.gen_range(1.0..3.0),
        x: rng.gen_range(0..wh.x as usize) as f32 - (wh.x / 2.0),
        y: rng.gen_range(0..wh.y as usize) as f32 - (wh.y / 2.0),
        age: 0,
        is_dead: false
    }
}

fn evolve_stars(_model: &mut Model) {
    for star in _model.stars.iter_mut() {
        star.age += 1;

        // twinkle
        star.w = _model.rng.gen_range(1.0..3.0);
        star.h = _model.rng.gen_range(1.0..3.0);

        // enforce half-life
        let p: f32 = 0.5f32.powf(star.age as f32 / STELLAR_HALF_LIFE as f32);
        if _model.rng.gen::<f32>() > p {
            star.is_dead = true;
        }
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    // remove dead stars
    _model.stars.retain(|s| !s.is_dead);

    // evolve existing stars
    evolve_stars(_model);

    // randomly choose parameters for a new star
    let star: Star = create_star(_app, &mut _model.rng);
    _model.stars.push(star);

    println!("stars: {}", _model.stars.len());
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    for star in _model.stars.iter() {
        draw.ellipse().color(WHITE).w(star.w).h(star.h).x_y(star.x, star.y);
    }

    draw.to_frame(app, &frame).unwrap();

    // Capture the frame!
    let file_path = captured_frame_path(app, &frame);
    app.main_window().capture_frame(file_path);
}

fn captured_frame_path(app: &App, frame: &Frame) -> std::path::PathBuf {
    // Create a path that we want to save this frame to.
    app.project_path()
        .expect("failed to locate `project_path`")
        // Capture all frames to a directory called `/<path_to_nannou>/nannou/simple_capture`.
        .join(app.exe_name().unwrap())
        // Name each file after the number of the frame.
        .join(format!("{:03}", frame.nth()))
        // The extension will be PNG. We also support tiff, bmp, gif, jpeg, webp and some others.
        .with_extension("png")
}

fn main() {
    nannou::app(model).update(update).run();
}
