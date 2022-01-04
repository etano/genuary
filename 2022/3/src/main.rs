use nannou::prelude::*;
// use ndarray::prelude::*;
// use ndarray::{Array, Ix2};
use rand::prelude::*;
use rand_pcg::Pcg64;

const SEED: u64 = 12345;
const WINDOW_X: u32 = 1000;
const WINDOW_Y: u32 = 1000;
const N_VISIBLE_STARS: u32 = 1000;
const MIN_STAR_PIXELS: f32 = 2.0;
const MAX_STAR_PIXELS: f32 = 200.0;
const MAX_TWINKLE_PIXELS: f32 = 5.0;
const MAX_WARP_SPEED: f32 = 500.0;
const MIN_WARP_SPEED: f32 = -50.0;
const WARP_SPEED_DELTA: f32 = 5.0;
const INITIAL_WARP_SPEED: f32 = MAX_WARP_SPEED;

#[derive(Copy, Clone)]
struct Star {
    w: f32,
    h: f32,
    x: f32,
    y: f32,
    angle: f32,
    age: u32,
    is_visible: bool
}

struct Model {
    _window: window::Id,
    wx: f32,
    wy: f32,
    rng: rand_pcg::Pcg64,
    stars: Vec<Star>,
    warp_speed: f32,
    warp_speed_delta: f32
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(WINDOW_X, WINDOW_Y)
        .view(view)
        .build()
        .unwrap();
    let rng = Pcg64::seed_from_u64(SEED);
    let wh = app.window_rect().wh();
    let wx = wh.x;
    let wy = wh.y;
    let stars: Vec<Star> = Vec::new();
    let warp_speed = INITIAL_WARP_SPEED;
    let warp_speed_delta = INITIAL_WARP_SPEED.signum() * WARP_SPEED_DELTA;

    Model {
        _window,
        wx,
        wy,
        rng,
        stars,
        warp_speed,
        warp_speed_delta
    }
}

fn create_star(_model: &mut Model) -> Star {
    let x: f32 = _model.rng.gen_range(-(_model.wx / 2.0)..(_model.wx / 2.0));
    let y: f32 = _model.rng.gen_range(-(_model.wy / 2.0)..(_model.wy / 2.0));
    let vx: f32 = _model.warp_speed * x / _model.wx as f32;
    let vy: f32 = _model.warp_speed * y / _model.wy as f32;
    Star{
        w: ((vx*vx + vy*vy).sqrt() + MIN_STAR_PIXELS).min(MAX_STAR_PIXELS),
        h: MIN_STAR_PIXELS,
        x: x,
        y: y,
        angle: (y/x).atan(),
        age: 0,
        is_visible: true
    }
}

fn evolve_stars(_model: &mut Model) {
    for star in _model.stars.iter_mut() {
        star.age += 1;

        // move
        let vx = _model.warp_speed * star.x / _model.wx;
        let vy = _model.warp_speed * star.y / _model.wy;
        star.x += vx;
        star.y += vy;
        star.angle = (star.y/star.x).atan();

        // twinkle
        star.w = ((vx.pow(2.0) + vy.pow(2.0)).sqrt() + MIN_STAR_PIXELS).min(MAX_STAR_PIXELS);
        star.h = MIN_STAR_PIXELS;

        // enforce death
        if star.x.abs() - star.w < _model.wx || star.y.abs() - star.w < _model.wy {
            star.is_visible = true;
        } else {
            star.is_visible = false;
        }
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    // remove dead stars
    _model.stars.retain(|s| s.is_visible);

    // randomly choose parameters for a new star
    while _model.stars.len() < N_VISIBLE_STARS as usize {
        let star: Star = create_star(_model);
        _model.stars.push(star);
    }

    // adjust warp speed
    if _model.warp_speed > MAX_WARP_SPEED || _model.warp_speed < MIN_WARP_SPEED {
        _model.warp_speed_delta *= -1.0;
    }
    _model.warp_speed += _model.warp_speed_delta;

    // evolve existing stars
    evolve_stars(_model);
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    for star in _model.stars.iter() {
        draw.ellipse().color(WHITE).w(star.w).h(star.h).x_y(star.x, star.y).rotate(star.angle);
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
