use nannou::image;
use nannou::prelude::*;
use nannou::text::FontSize;
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
const BETA_START: f32 = 0.01 * BETA_C;
const BETA_END: f32 = 3.0 * BETA_C;
const HOTSPOT_RADIUS: f32 = 50.0;
const N_EQUILIBRATION_STEPS: usize = 100;
const N_CHARACTER_STEPS: usize = 180;
const N_STATES: usize = 5;
const RGBAS: [[u8; 4]; N_STATES] = [
    [255, 48, 50, u8::MAX],
    [255, 96, 101, u8::MAX],
    [255, 141, 151, u8::MAX],
    [255, 192, 203, u8::MAX],
    [255, 255, 255, u8::MAX]
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
    hotspots: Vec<Point2>,
    characters: Vec<char>,
    n_equilibration_steps: usize,
    n_character_steps: usize,
    step: usize,
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
    let mut rng = Pcg64::seed_from_u64(SEED);
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
            a[[i, j]] = rng.gen_range(0..n_states) as i8;
            beta[[i, j]] = BETA_START;
        }
    }
    let hotspots: Vec<Point2> = Vec::new();
    let characters: Vec<char> = vec!['A', 'B', 'C'];
    let n_equilibration_steps: usize = N_EQUILIBRATION_STEPS;
    let n_character_steps: usize = N_CHARACTER_STEPS;
    let step: usize = 0;

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
        hotspots,
        characters,
        n_equilibration_steps,
        n_character_steps,
        step,
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

    if _model.step > _model.n_equilibration_steps {
        // get points from text
        let index = (_model.step - _model.n_equilibration_steps) / _model.n_character_steps;
        let c: &str = &_model.characters[index % _model.characters.len()].to_string();
        let rect = Rect::from_x_y_w_h(0.0, 0.0, _model.x1 - _model.x0, _model.y1 - _model.y0);
        let font_size: FontSize = ((72.0 / 96.0) * rect.h()) as FontSize;
        let text = text(c).font_size(font_size).center_justify().align_middle_y().build(rect);

        // shift for vertical alignment
        let mut min_y = 0.0;
        let mut max_y = 0.0;
        for event in text.path_events() {
            let (_, from_y) = event.from().to_tuple();
            let (_, to_y) = event.to().to_tuple();
            min_y = min_y.min(from_y).min(to_y);
            max_y = max_y.max(from_y).max(to_y);
        }
        let shift_y = (max_y + min_y) / 2.0;

        // fill in hotspots
        _model.hotspots.clear();
        for event in text.path_events() {
            let (from_x, from_y) = event.from().to_tuple();
            let (to_x, to_y) = event.to().to_tuple();
            let delta_x = to_x - from_x;
            let delta_y = to_y - from_y;
            let r = (delta_x*delta_x + delta_y*delta_y).sqrt();
            let dr = 10.0;
            let n_points: usize = (r / dr) as usize;
            let dx = delta_x / n_points as f32;
            let dy = delta_y / n_points as f32;
            let mut x = from_x;
            let mut y = from_y;
            for _ in 0..n_points {
                _model.hotspots.push(Point2::new(x, -y + shift_y));
                x += dx;
                y += dy;
            }
            _model.hotspots.push(Point2::new(to_x, -to_y + shift_y));
        }
    }

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
            _model.beta[[i, j]] = BETA_START * (1.0 - r / max_r).max(0.0) + BETA_END * (r / max_r).min(1.0);
        }
    }

    // step
    _model.step += 1
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
    if _model.step > _model.n_equilibration_steps {
        let file_path = captured_frame_path(app, &frame, _model.n_equilibration_steps);
        app.main_window().capture_frame(file_path);
    }
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
