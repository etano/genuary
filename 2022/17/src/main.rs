use nannou::image;
use nannou::prelude::*;
use ndarray::prelude::*;
use ndarray::{Array, Ix2};
use rand::prelude::*;
use rand_pcg::Pcg64;

const SEED: u64 = 12345;
const WINDOW_WIDTH: f32 = 600.0;
const WINDOW_HEIGHT: f32 = 600.0;
const SQUARE_WIDTH: f32 = 500.0;
const SQUARE_HEIGHT: f32 = 500.0;
const SPIN_WIDTH_X: f32 = 1.0;
const SPIN_WIDTH_Y: f32 = 1.0;
const BETA_C: f32 = 0.440686793509772; // (2.).sqrt().ln_1p() / 2.;
const BETA_START: f32 = 0.5 * BETA_C;
const BETA_END: f32 = 2.0 * BETA_C;
const N_STEPS: usize = 100;

struct Model {
    _window: window::Id,
    w_x: f32,
    w_y: f32,
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
    n_x: usize,
    n_y: usize,
    down_rgba: [u8; 4],
    up_rgba: [u8; 4],
    sideways_rgba: [u8; 4],
    a: Array<i8, Ix2>,
    rng: rand_pcg::Pcg64,
    beta: f32,
    beta_delta: f32,
    texture: wgpu::Texture,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        .view(view)
        .build()
        .unwrap();
    let rng = Pcg64::seed_from_u64(SEED);
    let beta: f32 = BETA_START;
    let beta_delta: f32 = (BETA_END - BETA_START) / N_STEPS as f32;
    let window = app.main_window();

    let offset_x = (WINDOW_WIDTH - SQUARE_WIDTH) / 2.0;
    let offset_y = (WINDOW_HEIGHT - SQUARE_HEIGHT) / 2.0;
    let x0: f32 = offset_x - 0.5*WINDOW_WIDTH;
    let y0: f32 = offset_y - 0.5*WINDOW_HEIGHT;
    let x1: f32 = SQUARE_WIDTH + offset_x - 0.5*WINDOW_WIDTH;
    let y1: f32 = SQUARE_HEIGHT + offset_y - 0.5*WINDOW_HEIGHT;

    let w_x: f32 = SPIN_WIDTH_X;
    let w_y: f32 = SPIN_WIDTH_Y;
    let n_x: usize = SQUARE_WIDTH as usize / w_x as usize;
    let n_y: usize = SQUARE_HEIGHT as usize / w_y as usize;

    let down_rgba: [u8; 4] = [255, 96, 101, u8::MAX];
    let up_rgba: [u8; 4] = [255, 141, 151, u8::MAX];
    let sideways_rgba: [u8; 4] = [255, 192, 203, u8::MAX];

    println!("w_x {}, w_y {}, n_x {}, n_y {}", w_x, w_y, n_x, n_y);
    let a = Array::<i8, Ix2>::ones((n_x, n_y).f());
    let texture = wgpu::TextureBuilder::new()
        .size([WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32])
        .format(wgpu::TextureFormat::Rgba8Unorm)
        .usage(wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING)
        .build(window.device());

    Model {
        _window,
        w_x,
        w_y,
        x0,
        x1,
        y0,
        y1,
        n_x,
        n_y,
        a,
        down_rgba,
        up_rgba,
        sideways_rgba,
        rng,
        beta,
        beta_delta,
        texture,
    }
}

fn compute_energy(a: &Array<i8, Ix2>, i: usize, j: usize, n_x: usize, n_y: usize) -> f32 {
    let neighbors = vec![
        a[[(i + n_x - 1) % n_x, j]],
        a[[(i + 1) % n_x, j]],
        a[[i, (j + n_y - 1) % n_y]],
        a[[i, (j + 1) % n_y]]
    ];
    let aij = a[[i, j]];
    let mut energy = 0.0;
    for neighbor in neighbors.iter() {
        if aij == *neighbor {
            energy -= 1.0;
        } else {
            energy += 1.0;
        }
    }
    energy as f32
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    let n_x = _model.n_x;
    let n_y = _model.n_y;
    let beta = _model.beta;
    for _ in 0..(n_x * n_y) {
        let i: usize = _model.rng.gen_range(0..n_x);
        let j: usize = _model.rng.gen_range(0..n_y);

        // compute energy
        let old_energy = compute_energy(&_model.a, i, j, n_x, n_y);

        // make a move
        let old_aij = _model.a[[i, j]];
        _model.a[[i, j]] = _model.rng.gen_range(-1..=1);
        // println!("aij: {}", _model.a[[i, j]]);

        // compute energy
        let new_energy = compute_energy(&_model.a, i, j, n_x, n_y);

        // flip a coin and reject if condition is met
        if _model.rng.gen::<f32>() > (beta * (old_energy - new_energy) as f32).exp() {
            _model.a[[i, j]] = old_aij;
        }
    }

    // increment beta
    if _model.beta > BETA_END || _model.beta < BETA_START {
        _model.beta_delta *= -1.;
    }
    _model.beta += _model.beta_delta;
    // println!("beta {}, beta_end {}, beta_start {}, beta_delta {}", _model.beta, BETA_END, BETA_START, _model.beta_delta);
}

fn get_rgba(pixel_x: usize, pixel_y: usize, _model: &Model) -> [u8; 4] {
    let x = pixel_x as f32 - 0.5 * WINDOW_WIDTH;
    let y = pixel_y as f32 - 0.5 * WINDOW_HEIGHT;
    if _model.x0 <= x && x < _model.x1 && _model.y0 <= y && y < _model.y1 {
        // println!("pixel_x: {}, pixel_y: {}, x: {}, y: {}", pixel_x, pixel_y, x, y);
        let i: usize = ((x - _model.x0) / _model.w_x) as usize;
        let j: usize = ((y - _model.y0) / _model.w_y) as usize;
        let val = _model.a[[i, j]];
        if val == -1 {
            _model.down_rgba
        } else if val == 0 {
            _model.sideways_rgba
        } else {
            _model.up_rgba
        }
    } else {
        let bg: [u8; 4] = [0, 0, 0, u8::MAX];
        bg
    }
}

fn view(app: &App, _model: &Model, frame: Frame) {
    frame.clear(BLACK);

    let image = image::ImageBuffer::from_fn(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, |i, j| {
        let rgba = get_rgba(i as usize, j as usize, _model);
        nannou::image::Rgba(rgba)
    });

    let flat_samples = image.as_flat_samples();
    _model.texture.upload_data(
        app.main_window().device(),
        &mut *frame.command_encoder(),
        &flat_samples.as_slice(),
    );

    let draw = app.draw();
    draw.texture(&_model.texture);
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
