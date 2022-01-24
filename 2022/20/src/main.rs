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
const SPIN_WIDTH_X: f32 = 10.0;
const SPIN_WIDTH_Y: f32 = 10.0;
const BETA_C: f32 = 0.440686793509772; // (2.).sqrt().ln_1p() / 2.;
const BETA_START: f32 = 0.01 * BETA_C;
const BETA_END: f32 = 3.0 * BETA_C;
const N_EQUILIBRATION_STEPS: usize = 100;
const N_STATES: usize = 3;
const RGBAS: [[u8; 4]; N_STATES] = [
    [255, 141, 151, u8::MAX],
    [255, 192, 203, u8::MAX],
    [255, 255, 255, u8::MAX]
];

struct Model {
    _window: window::Id,
    w_x: f32,
    w_y: f32,
    x0: f32,
    // x1: f32,
    y0: f32,
    // y1: f32,
    n_x: usize,
    n_y: usize,
    n_states: usize,
    rgbas: [[u8; 4]; N_STATES],
    a: Array<i8, Ix2>,
    beta: f32,
    beta_delta: f32,
    step: usize,
    rng: rand_pcg::Pcg64,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        .view(view)
        .build()
        .unwrap();
    let rng = Pcg64::seed_from_u64(SEED);

    let offset_x = (WINDOW_WIDTH - SQUARE_WIDTH) / 2.0;
    let offset_y = (WINDOW_HEIGHT - SQUARE_HEIGHT) / 2.0;
    let x0: f32 = offset_x - 0.5*WINDOW_WIDTH;
    let y0: f32 = offset_y - 0.5*WINDOW_HEIGHT;
    // let x1: f32 = SQUARE_WIDTH + offset_x - 0.5*WINDOW_WIDTH;
    // let y1: f32 = SQUARE_HEIGHT + offset_y - 0.5*WINDOW_HEIGHT;

    let w_x: f32 = SPIN_WIDTH_X;
    let w_y: f32 = SPIN_WIDTH_Y;
    let n_x: usize = SQUARE_WIDTH as usize / w_x as usize;
    let n_y: usize = SQUARE_HEIGHT as usize / w_y as usize;

    let n_states: usize = N_STATES;
    let rgbas: [[u8; 4]; N_STATES] = RGBAS;

    println!("w_x {}, w_y {}, n_x {}, n_y {}", w_x, w_y, n_x, n_y);
    let a = Array::<i8, Ix2>::zeros((n_x, n_y).f());
    let beta = BETA_START;
    let beta_delta: f32 = (BETA_END - BETA_START) / N_EQUILIBRATION_STEPS as f32;
    let step: usize = 0;

    Model {
        _window,
        w_x,
        w_y,
        x0,
        // x1,
        y0,
        // y1,
        n_x,
        n_y,
        n_states,
        rgbas,
        a,
        beta,
        beta_delta,
        step,
        rng,
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
        if _model.rng.gen::<f32>().ln() > (_model.beta * (old_energy - new_energy) as f32) {
            _model.a[[i, j]] = old_aij;
        }
    }

    // increment beta
    if _model.beta > BETA_END || _model.beta < BETA_START {
        _model.beta_delta *= -1.;
    }
    _model.beta += _model.beta_delta;

    // step
    _model.step += 1
}

fn get_color(_model: &Model, index: usize) -> Rgba<u8> {
    let color = rgba(
        _model.rgbas[index][0],
        _model.rgbas[index][1],
        _model.rgbas[index][2],
        _model.rgbas[index][3],
    );
    color
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    for i in 0.._model.n_x {
        for j in 0.._model.n_y {
            let val = _model.a[[i, j]] as usize;
            let n_points = match val {
                0 => 2,
                1 => 3,
                2 => 5,
                _ => 3
            };
            let points = (0..=360).step_by(360 / n_points).map(|theta| {
               let radian = deg_to_rad(theta as f32);
               let x = _model.x0 + (i as f32 + 0.5 + 0.5 * radian.sin()) * _model.w_x;
               let y = _model.y0 + (j as f32 + 0.5 + 0.5 * radian.cos()) * _model.w_y;
               pt2(x,y)
            });
            draw.polygon()
                .no_fill()
                .stroke_color(get_color(_model, val))
                .stroke_weight(2.0)
                .points(points);
        }
    }
    draw.to_frame(app, &frame).unwrap();

    // Capture the frame!
    let file_path = captured_frame_path(app, &frame, 0);
    app.main_window().capture_frame(file_path);
}

fn captured_frame_path(app: &App, frame: &Frame, offset: usize) -> std::path::PathBuf {
    // Create a path that we want to save this frame to.
    app.project_path()
        .expect("failed to locate `project_path`")
        // Capture all frames to a directory called `/<path_to_nannou>/nannou/simple_capture`.
        .join(app.exe_name().unwrap())
        // Name each file after the number of the frame.
        .join(format!("{:03}", frame.nth() - offset as u64))
        // The extension will be PNG. We also support tiff, bmp, gif, jpeg, webp and some others.
        .with_extension("png")
}

fn main() {
    nannou::app(model).update(update).run();
}
