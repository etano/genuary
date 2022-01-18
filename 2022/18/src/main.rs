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
const HOTSPOT_PATH_RADIUS: f32 = 150.0;
const HOTSPOT_RADIUS: f32 = 50.0;
const N_HOTSPOTS: usize = 360;
const N_STEPS: usize = 300;
const N_STATES: usize = 2;
const RGBAS: [[u8; 4]; N_STATES] = [
    [0, 0, 0, u8::MAX],
    [u8::MAX, u8::MAX, u8::MAX, u8::MAX]
];

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
    n_states: usize,
    rgbas: [[u8; 4]; N_STATES],
    a: Array<i8, Ix2>,
    beta: Array<f32, Ix2>,
    global_beta: f32,
    beta_delta: f32,
    hotspots: Vec<[f32; 3]>,
    n_steps: usize,
    rng: rand_pcg::Pcg64,
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

    let n_states: usize = N_STATES;
    let rgbas: [[u8; 4]; N_STATES] = RGBAS;

    println!("w_x {}, w_y {}, n_x {}, n_y {}", w_x, w_y, n_x, n_y);
    let mut a = Array::<i8, Ix2>::zeros((n_x, n_y).f());
    let mut beta = Array::<f32, Ix2>::zeros((n_x, n_y).f());
    for i in 0..n_x {
        for j in 0..n_y {
            a[[i, j]] = 0;
            beta[[i, j]] = BETA_END;
        }
    }
    let global_beta = BETA_END;
    let beta_delta: f32 = (BETA_END - BETA_START) / N_STEPS as f32;
    let mut hotspots: Vec<[f32; 3]> = Vec::new();
    let mut theta: f32 = 0.0;
    let x_mid = (x1 + x0) / 2.0;
    let y_mid = (y1 + y0) / 2.0;
    for _ in 0..N_HOTSPOTS {
        let x = HOTSPOT_PATH_RADIUS * theta.cos() + x_mid;
        let y = HOTSPOT_PATH_RADIUS * theta.sin() + y_mid;
        hotspots.push([x, y, theta]);
        theta += 2.0 * PI / N_HOTSPOTS as f32;
    }
    let n_steps: usize = N_STEPS;

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
        n_states,
        rgbas,
        a,
        beta,
        global_beta,
        beta_delta,
        hotspots,
        n_steps,
        rng,
        texture,
    }
}

fn compute_energy(a: &Array<i8, Ix2>, i: usize, j: usize, n_x: usize, n_y: usize) -> f32 {
    let aij = a[[i, j]];
    let nn = [
        a[[(i + n_x - 1) % n_x, j]],
        a[[(i + 1) % n_x, j]],
        a[[i, (j + n_y - 1) % n_y]],
        a[[i, (j + 1) % n_y]]
    ];
    let n_same_neighbors = nn.iter().filter(|&n| aij == *n).count();
    let energy: f32 = 4.0 - 2.0 * (n_same_neighbors as f32);
    energy
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    let n_x = _model.n_x;
    let n_y = _model.n_y;
    for _ in 0..(n_x * n_y) {
        let i: usize = _model.rng.gen_range(0..n_x);
        let j: usize = _model.rng.gen_range(0..n_y);

        // compute energy
        let old_energy = compute_energy(&_model.a, i, j, n_x, n_y);

        // make a move
        let old_aij = _model.a[[i, j]];
        _model.a[[i, j]] = _model.rng.gen_range(0.._model.n_states) as i8;

        // compute energy
        let new_energy = compute_energy(&_model.a, i, j, n_x, n_y);

        // flip a coin and reject if condition is met
        if _model.rng.gen::<f32>().ln() > (_model.beta[[i, j]] * (old_energy - new_energy) as f32) {
            _model.a[[i, j]] = old_aij;
        }
    }

    // evolve hot patches
    let delta_theta = 2.0 * PI / (_model.n_steps as f32);
    let x_mid = (_model.x1 + _model.x0) / 2.0;
    let y_mid = (_model.y1 + _model.y0) / 2.0;
    for hs in _model.hotspots.iter_mut() {
        let theta = hs[2] + delta_theta;
        hs[0] = HOTSPOT_PATH_RADIUS * theta.cos() + x_mid;
        hs[1] = HOTSPOT_PATH_RADIUS * theta.sin() + y_mid;
        hs[2] = theta;
    }

    // increment global beta
    if _model.global_beta > BETA_END || _model.global_beta < BETA_START {
        _model.beta_delta *= -1.;
    }
    _model.global_beta += _model.beta_delta;

    // compute new beta
    for i in 0..n_x {
        for j in 0..n_y {
            let x = i as f32 * _model.w_x + _model.x0;
            let y = j as f32 * _model.w_y + _model.y0;
            let max_r: f32 = HOTSPOT_RADIUS;
            let mut r: f32 = max_r;
            for hs in _model.hotspots.iter() {
                r = ((hs[0] - x).pow(2) as f32 + (hs[1] - y).pow(2) as f32).sqrt().min(r);
            }
            _model.beta[[i, j]] = BETA_START * (1.0 - r / max_r).max(0.0) + _model.global_beta * (r / max_r).min(1.0);
        }
    }
}

fn get_rgba(pixel_x: usize, pixel_y: usize, _model: &Model) -> [u8; 4] {
    let x = pixel_x as f32 - 0.5 * WINDOW_WIDTH;
    let y = pixel_y as f32 - 0.5 * WINDOW_HEIGHT;
    if _model.x0 <= x && x < _model.x1 && _model.y0 <= y && y < _model.y1 {
        let i: usize = ((x - _model.x0) / _model.w_x) as usize;
        let j: usize = ((y - _model.y0) / _model.w_y) as usize;
        let val = _model.a[[i, j]] as usize;
        let rgba = _model.rgbas[val];
        rgba
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
