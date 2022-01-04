use nannou::prelude::*;
use rand::prelude::*;
use rand_pcg::Pcg64;

const SEED: u64 = 12345;
const WINDOW_X: usize = 1000;
const WINDOW_Y: usize = 1000;
const N_VISIBLE_LINES: usize = 10000;

#[derive(Copy, Clone)]
struct Line {
    start: Point2,
    end: Point2,
    is_visible: bool
}

struct Model {
    _window: window::Id,
    lines: Vec<Line>,
    rng: rand_pcg::Pcg64,
    whx: f32,
    why: f32
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(WINDOW_X as u32, WINDOW_Y as u32)
        .view(view)
        .build()
        .unwrap();
    let mut rng = Pcg64::seed_from_u64(SEED);
    let whx = WINDOW_X as f32;
    let why = WINDOW_Y as f32;

    let mut lines: Vec<Line> = Vec::new();
    while lines.len() < N_VISIBLE_LINES {
        let start_x = rng.gen_range(-whx/2.0..whx/2.0);
        let start_y = rng.gen_range(-why/2.0..why/2.0);
        let line: Line = Line{
            start: Point2::new(start_x, start_y),
            end: Point2::new(start_x, start_y),
            is_visible: true
        };
        lines.push(line);
    }

    Model {
        _window,
        lines,
        rng,
        whx,
        why
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    // remove lines
    _model.lines.retain(|l| l.is_visible);

    // add lines
    //while _model.lines.len() < N_VISIBLE_LINES as usize {
    for _ in 0..100 {
        let start_x = _model.rng.gen_range(-_model.whx/2.0.._model.whx/2.0);
        let start_y = _model.rng.gen_range(-_model.why/2.0.._model.why/2.0);
        let line: Line = Line{
            start: Point2::new(start_x, start_y),
            end: Point2::new(start_x, start_y),
            is_visible: true
        };
        _model.lines.push(line);
    }

    // evolve lines
    for line in _model.lines.iter_mut() {
        let start_x = line.start.to_array()[0];
        let start_y = line.start.to_array()[1];
        let end_x = line.end.to_array()[0];
        let end_y = line.end.to_array()[1];

        let px: f32 = (start_x + end_x) / 2.0;
        let py: f32 = (start_y + end_y) / 2.0;

        let x: f32 = 0.5 + px / _model.whx;
        let y: f32 = 0.5 + py / _model.why;
        let xp: f32 = 2.0*x*x + y;
        let yp: f32 = x*x;
        let vx: f32 = xp - x;
        let vy: f32 = yp - y;
        let theta = (vy/vx).atan();
        let r = 100.0 * (vx*vx + vy*vy).sqrt() / 2.0;

        let xa = (px + vx) - 0.5 * r * theta.cos();
        let ya = (py + vy) - 0.5 * r * theta.sin();
        let xb = (px + vx) + 0.5 * r * theta.cos();
        let yb = (py + vy) + 0.5 * r * theta.sin();

        let new_start_x = if vx < 0.0 { xb } else { xa };
        let new_end_x = if vx < 0.0 { xa } else { xb };
        let new_start_y = if vx < 0.0 { yb } else { ya };
        let new_end_y = if vx < 0.0 { ya } else { yb };

        line.start = Point2::new(new_start_x, new_start_y);
        line.end = Point2::new(new_end_x, new_end_y);

        // enforce death
        if new_end_x.abs() < _model.whx/2.0 || new_end_y.abs() < _model.why/2.0 {
            line.is_visible = true;
        } else {
            line.is_visible = false;
        }
    }
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    for line in _model.lines.iter() {
        draw.line().color(PINK).weight(1.0).points(line.start, line.end);
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
