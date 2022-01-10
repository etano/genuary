use nannou::prelude::*;
use rand::prelude::*;
use rand_pcg::Pcg64;

const SEED: u64 = 123456;
const WINDOW_WIDTH: usize = 1000;
const WINDOW_HEIGHT: usize = 1000;
const SQUARE_WIDTH: f32 = 500.0;
const SQUARE_HEIGHT: f32 = 500.0;
const STEP_SIZE: f32 = 10.0;
const TARGET_RADIUS: f32 = 5.0;
const SEARCH_RADIUS: f32 = 5.0;
const N_STEPS_PER_SWEEP: usize = 1;

struct Model {
    _window: window::Id,
    target: Point2,
    search: Point2,
    path: Vec<Point2>,
    target_found: bool,
    beta: f32,
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
    rng: rand_pcg::Pcg64
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        .view(view)
        .build()
        .unwrap();
    let mut rng = Pcg64::seed_from_u64(SEED);
    let window_width = WINDOW_WIDTH as f32;
    let window_height = WINDOW_HEIGHT as f32;

    let offset_x = (window_width - SQUARE_WIDTH) / 2.0;
    let offset_y = (window_height - SQUARE_HEIGHT) / 2.0;
    let x0: f32 = offset_x - 0.5*window_width;
    let y0: f32 = offset_y - 0.5*window_height;
    let x1: f32 = SQUARE_WIDTH + offset_x - 0.5*window_width;
    let y1: f32 = SQUARE_HEIGHT + offset_y - 0.5*window_height;
    let target: Point2 = Point2::new(rng.gen_range(x0..x1), rng.gen_range(y0..y1));
    let search: Point2 = Point2::new(rng.gen_range(x0..x1), rng.gen_range(y0..y1));
    let path: Vec<Point2> = Vec::new();
    let target_found: bool = false;
    let beta = 1.0;

    Model {
        _window,
        target,
        search,
        path,
        target_found,
        beta,
        x0,
        x1,
        y0,
        y1,
        rng
    }
}

fn compute_energy(target: &Point2, search: &Point2) -> f32 {
    let target_x = target[0];
    let target_y = target[1];
    let search_x = search[0];
    let search_y = search[1];
    let energy = (target_x - search_x).pow(2.0) + (target_y - search_y).pow(2.0);
    energy
}

fn put_in_box(point: &mut Point2, x0: f32, x1: f32, y0: f32, y1: f32) {
    let model_width = x1 - x0;
    let model_height = y1 - y0;
    while point[0] > x1 - SEARCH_RADIUS {
        point[0] -= model_width;
    }
    while point[0] < x0 + SEARCH_RADIUS {
        point[0] += model_width;
    }
    while point[1] > y1 - SEARCH_RADIUS {
        point[1] -= model_height;
    }
    while point[1] < y0 + SEARCH_RADIUS {
        point[1] += model_height;
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    _model.path.push(_model.search);

    // check if search reached target
    let target_x = _model.target[0];
    let target_y = _model.target[1];
    let search_x = _model.search[0];
    let search_y = _model.search[1];
    let r = ((target_x - search_x).pow(2.0) + (target_y - search_y).pow(2.0)).sqrt();
    if r <= TARGET_RADIUS + SEARCH_RADIUS {
        _model.target_found = true;
        _model.path = Vec::new();
        _model.target = Point2::new(
            _model.rng.gen_range(_model.x0.._model.x1),
            _model.rng.gen_range(_model.y0.._model.y1)
        );
        _model.search = Point2::new(
            _model.rng.gen_range(_model.x0.._model.x1),
            _model.rng.gen_range(_model.y0.._model.y1)
        );
    } else {
        _model.target_found = false;
        // update search
        for _ in 0..N_STEPS_PER_SWEEP {
            let theta = _model.rng.gen_range(0.0..2.0*PI);
            let dx = STEP_SIZE * theta.cos();
            let dy = STEP_SIZE * theta.sin();

            let energy0: f32 = compute_energy(&_model.target, &_model.search);
            _model.search[0] += dx;
            _model.search[1] += dy;
            put_in_box(&mut _model.search, _model.x0, _model.x1, _model.y0, _model.y1);
            let energy1: f32 = compute_energy(&_model.target, &_model.search);

            if _model.beta * (energy0 - energy1).exp() < _model.rng.gen_range(0.0..1.0) {
                // reject
                _model.search[0] -= dx;
                _model.search[1] -= dy;
                put_in_box(&mut _model.search, _model.x0, _model.x1, _model.y0, _model.y1);
            }
        }
    }
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    draw.rect().color(BLACK).stroke(PINK).stroke_weight(1.0).x(0.0).y(0.0).w(_model.x1 - _model.x0).h(_model.y1 - _model.y0);

    if _model.path.len() > 0 {
        for i in 0.._model.path.len() - 1 {
            draw.line().color(PINK).weight(1.0).start(_model.path[i]).end(_model.path[i+1]);
        }
    }
    draw.ellipse().no_fill().stroke_color(PINK).stroke_weight(3.0).radius(TARGET_RADIUS).xy(_model.target);
    draw.ellipse().color(PINK).radius(SEARCH_RADIUS).xy(_model.search);
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
