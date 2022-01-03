use nannou::prelude::*;
// use ndarray::prelude::*;
// use ndarray::{Array, Ix2};
use rand::prelude::*;
use rand_pcg::Pcg64;

const SEED: u64 = 12345;
const WINDOW_X: u32 = 1000;
const WINDOW_Y: u32 = 1000;

#[derive(Copy, Clone)]
enum StarStage {
    BIRTH,
    YOUTH,
    MATURITY,
    DEATH
}

#[derive(Copy, Clone)]
struct Star {
    x: f32,
    y: f32,
    stage: StarStage,
    age: u32
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
    let rng = Pcg64::seed_from_u64(SEED);
    let stars: Vec<Star> = Vec::new();

    Model {
        _window,
        rng,
        stars
    }
}

fn evolve_star(star: &mut Star) {
    star.age += 1;
    // println!("{}", star.age);
    match star.stage {
        StarStage::BIRTH => {
            if star.age > 10 {
                star.stage = StarStage::YOUTH;
            }
        },
        StarStage::YOUTH => { star.stage = StarStage::MATURITY; },
        StarStage::MATURITY => { star.stage = StarStage::DEATH; },
        StarStage::DEATH => { star.stage = StarStage::DEATH; }
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    // remove dead stars
    _model.stars.retain(|s| !matches!(s.stage, StarStage::DEATH));

    // evolve existing stars
    for star in _model.stars.iter_mut() {
        evolve_star(star);
    }

    // randomly choose coordinates for a new star
    let wh = _app.window_rect().wh();
    let x: f32 = _model.rng.gen_range(0..wh.x as usize) as f32 - (wh.x / 2.0);
    let y: f32 = _model.rng.gen_range(0..wh.y as usize) as f32 - (wh.y / 2.0);

    println!("x, y: {}, {}", x, y);
    _model.stars.push(Star{x: x, y: y, stage: StarStage::BIRTH, age: 0});

    println!("stars: {}", _model.stars.len());
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    for star in _model.stars.iter() {
        draw.ellipse().color(WHITE).w(2.0).h(2.0).x_y(star.x, star.y);
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
