use nannou::lyon;
use nannou::prelude::*;
use rand::prelude::*;
use rand_pcg::Pcg64;

const SEED: u64 = 123456;
const WINDOW_WIDTH: usize = 1000;
const WINDOW_HEIGHT: usize = 1000;
const SQUARE_WIDTH: f32 = 500.0;
const SQUARE_HEIGHT: f32 = 500.0;
const N_POINTS: usize = 100;
const N_POINTS_PER_SWEEP: usize = 3;
const N_SWEEPS: usize = 50;
const N_STEPS_PER_SWEEP: usize = 10;
const STEP_SIZE: f32 = 1.0;

struct Model {
    _window: window::Id,
    points: Vec<Point2>,
    relax_mode: bool,
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
    sweep: usize,
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

    let offset_x = (window_width - SQUARE_WIDTH) / 2.0;
    let offset_y = (window_height - SQUARE_HEIGHT) / 2.0;
    let x0: f32 = offset_x - 0.5*window_width;
    let y0: f32 = offset_y - 0.5*window_height;
    let x1: f32 = SQUARE_WIDTH + offset_x - 0.5*window_width;
    let y1: f32 = SQUARE_HEIGHT + offset_y - 0.5*window_height;
    let points: Vec<Point2> = Vec::new();
    let relax_mode: bool = false;
    let sweep: usize = 0;

    Model {
        _window,
        points,
        relax_mode,
        x0,
        x1,
        y0,
        y1,
        sweep,
        rng
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    for point in _model.points.iter_mut() {
        for _ in 0..N_STEPS_PER_SWEEP {
            let theta = _model.rng.gen_range(0.0..2.0*PI);
            let dx = STEP_SIZE * theta.cos();
            let dy = STEP_SIZE * theta.sin();
            point[0] += dx;
            point[1] += dy;
        }
    }
    if _model.relax_mode {
        // sweep
        _model.sweep += 1;
        if _model.sweep > N_SWEEPS {
            _model.points = Vec::new();
            _model.sweep = 0;
            _model.relax_mode = false;
        }
    } else {
        // add points
        for _ in 0..N_POINTS_PER_SWEEP {
            let x = _model.rng.gen_range(_model.x0.._model.x1);
            let y = _model.rng.gen_range(_model.y0.._model.y1);
            let point = Point2::new(x, y);
            _model.points.push(point);
        }

        if _model.points.len() >= N_POINTS {
            _model.relax_mode = true;
        }
    }

}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    let n_points = _model.points.len();
    if n_points > 2 {
        let mut builder = nannou::geom::path::Builder::new().with_svg();
        for i in 0..n_points-2 {
            builder.move_to(lyon::math::point(_model.points[i][0], _model.points[i][1]));
            builder.cubic_bezier_to(
                lyon::math::point(_model.points[i][0], _model.points[i][1]),
                lyon::math::point(_model.points[i+1][0], _model.points[i+1][1]),
                lyon::math::point(_model.points[i+2][0], _model.points[i+2][1]),
            );
        }
        builder.close();
        let path = builder.build();

        draw.path().fill().color(PINK).events(path.iter());
        draw.to_frame(app, &frame).unwrap();
    }

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
