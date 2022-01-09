use nannou::prelude::*;
use rand::prelude::*;
use rand::distributions::{Distribution, Standard};
use rand_pcg::Pcg64;

const SEED: u64 = 123456;
const WINDOW_WIDTH: usize = 1000;
const WINDOW_HEIGHT: usize = 1000;
const SQUARE_WIDTH: f32 = 500.0;
const SQUARE_HEIGHT: f32 = 500.0;
const LINE_LENGTH: f32 = 50.0;
const N_LINES_PER_STEP: usize = 100;
const N_LINES: usize = 2000;

#[derive(Copy, Clone)]
struct Line {
    start: Point2,
    end: Point2
}

#[derive(Copy, Clone)]
enum Mode {
    Horizontal,
    Vertical,
    DiagonalLeft,
    DiagonalRight
}

impl Distribution<Mode> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Mode {
        match rng.gen_range(0..4) {
            0 => Mode::Horizontal,
            1 => Mode::Vertical,
            2 => Mode::DiagonalLeft,
            3 => Mode::DiagonalRight,
            _ => Mode::Horizontal
        }
    }
}

struct Model {
    _window: window::Id,
    lines: Vec<Line>,
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
    add_lines: bool,
    mode0: Mode,
    mode1: Mode,
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

    let lines: Vec<Line> = Vec::new();
    let offset_x = (window_width - SQUARE_WIDTH) / 2.0;
    let offset_y = (window_height - SQUARE_HEIGHT) / 2.0;
    let x0: f32 = offset_x - 0.5*window_width;
    let y0: f32 = offset_y - 0.5*window_height;
    let x1: f32 = SQUARE_WIDTH + offset_x - 0.5*window_width;
    let y1: f32 = SQUARE_HEIGHT + offset_y - 0.5*window_height;
    let add_lines: bool = true;
    let mode0: Mode = rng.gen();
    let mode1: Mode = rng.gen();

    Model {
        _window,
        lines,
        x0,
        x1,
        y0,
        y1,
        add_lines,
        mode0,
        mode1,
        rng
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    // determine mode
    if _model.lines.len() >= N_LINES {
        _model.add_lines = false;
    } else if _model.lines.len() == 0 {
        _model.add_lines = true;
        _model.mode0 = _model.rng.gen();
        _model.mode1 = _model.rng.gen();
    }

    // add lines
    if _model.add_lines {
        let half_line = 0.5*LINE_LENGTH;
        let half_line_projection = 0.5.sqrt()*half_line;
        for _ in 0..N_LINES_PER_STEP {
            let mut x0 = _model.x0;
            let mut x1 = _model.x1;
            let mut y0 = _model.y0;
            let mut y1 = _model.y1;
            let mut line = Line{start: Point2::new(x0, y0), end: Point2::new(x1, y1)};

            let use_mode0: bool = _model.rng.gen_bool(0.5);
            let mode = if use_mode0 { _model.mode0 } else { _model.mode1 };
            match mode {
                Mode::Horizontal => {
                    x0 += half_line;
                    x1 -= half_line;
                }
                Mode::Vertical => {
                    y0 += half_line;
                    y1 -= half_line;
                }
                Mode::DiagonalLeft => {
                    x0 += half_line_projection;
                    x1 -= half_line_projection;
                    y0 += half_line_projection;
                    y1 -= half_line_projection;
                }
                Mode::DiagonalRight => {
                    x0 += half_line_projection;
                    x1 -= half_line_projection;
                    y0 += half_line_projection;
                    y1 -= half_line_projection;
                }
            }

            let x = _model.rng.gen_range(x0..x1);
            let y = _model.rng.gen_range(y0..y1);

            match mode {
                Mode::Horizontal => {
                    line.start = Point2::new(x-half_line, y);
                    line.end = Point2::new(x+half_line, y);
                }
                Mode::Vertical => {
                    line.start = Point2::new(x,y-half_line);
                    line.end = Point2::new(x,y+half_line);
                }
                Mode::DiagonalLeft => {
                    line.start = Point2::new(x-half_line_projection,y-half_line_projection);
                    line.end = Point2::new(x+half_line_projection,y+half_line_projection);
                }
                Mode::DiagonalRight => {
                    line.start = Point2::new(x-half_line_projection,y+half_line_projection);
                    line.end = Point2::new(x+half_line_projection,y-half_line_projection);
                }
            }

            _model.lines.push(line);
        }
    } else {
        for _ in 0..N_LINES_PER_STEP {
            let i = _model.rng.gen_range(0.._model.lines.len());
            _model.lines.remove(i);
        }
    }
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    for line in _model.lines.iter() {
        draw.line().color(PINK).weight(1.0).start(line.start).end(line.end);
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
