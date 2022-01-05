use nannou::prelude::*;
use rand::prelude::*;
use rand_pcg::Pcg64;

const SEED: u64 = 12345;
const WINDOW_WIDTH: usize = 1000;
const WINDOW_HEIGHT: usize = 1000;
const SQUARE_WIDTH: usize = 500;
const SQUARE_HEIGHT: usize = 500;
const N_X: usize = 100;
const N_Y: usize = 100;
const N_STEPS: usize = 100;

#[derive(Copy, Clone)]
struct Particle {
    x0: f32,
    y0: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32
}

struct Model {
    _window: window::Id,
    particles: Vec<Particle>,
    window_width: f32,
    window_height: f32,
    step: usize,
    n_steps: usize,
    rng: rand_pcg::Pcg64
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        .view(view)
        .build()
        .unwrap();
    let rng = Pcg64::seed_from_u64(SEED);
    let window_width = WINDOW_WIDTH as f32;
    let window_height = WINDOW_HEIGHT as f32;

    let mut particles: Vec<Particle> = Vec::new();
    let offset_x = (window_width - SQUARE_WIDTH as f32) / 2.0;
    let offset_y = (window_height - SQUARE_HEIGHT as f32) / 2.0;
    let w = SQUARE_WIDTH as f32 / N_X as f32;
    let h = SQUARE_HEIGHT as f32 / N_Y as f32;
    for i in 0..N_X {
        for j in 0..N_Y {
            let particle = Particle{
                x0: i as f32 * w + offset_x - (window_width as f32)/2.0,
                y0: j as f32 * h + offset_y - (window_height as f32)/2.0,
                x: i as f32 * w + offset_x - (window_width as f32)/2.0,
                y: j as f32 * h + offset_y - (window_height as f32)/2.0,
                w: w,
                h: h
            };
            particles.push(particle);
        }
    }
    let step: usize = 0;
    let n_steps = N_STEPS;

    Model {
        _window,
        particles,
        window_width,
        window_height,
        step,
        n_steps,
        rng
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    // evolve particles
    let wave = 2.0 * (2.0 * PI * _model.step as f32 / _model.n_steps as f32).cos().abs();
    for particle in _model.particles.iter_mut() {
        let x = particle.x/_model.window_width + 0.5;
        let y = particle.y/_model.window_height + 0.5;
        if y + x > wave {
            particle.x += _model.rng.gen_range(-10.0..10.0);
            particle.y += _model.rng.gen_range(-10.0..10.0);
        } else {
            particle.x = particle.x0;
            particle.y = particle.y0;
        }
    }
    println!("wave {}", wave);

    // step
    _model.step += 1;
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    for particle in _model.particles.iter() {
        draw.ellipse().color(PINK).w(particle.w).h(particle.h).x(particle.x).y(particle.y);
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
