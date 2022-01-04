use nannou::prelude::*;
use rand::prelude::*;
use rand_pcg::Pcg64;

const SEED: u64 = 12345;
const WINDOW_X: usize = 1000;
const WINDOW_Y: usize = 1000;
const SPIN_WIDTH_X: usize = 20;
const SPIN_WIDTH_Y: usize = 20;
const N_HIGHLIGHT_STEPS: usize = 50;

#[derive(Copy, Clone)]
struct Line {
    start: Point2,
    end: Point2,
    highlight_start: Point2,
    highlight_end: Point2,
    step: usize,
    n_highlight_steps: usize
}

struct Model {
    _window: window::Id,
    lines: Vec<Line>,
    rng: rand_pcg::Pcg64
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(WINDOW_X as u32, WINDOW_Y as u32)
        .view(view)
        .build()
        .unwrap();
    let rng = Pcg64::seed_from_u64(SEED);
    let window = app.main_window();
    let wh = window.rect();
    let n_x: usize = wh.w() as usize / SPIN_WIDTH_X as usize;
    let n_y: usize = wh.h() as usize / SPIN_WIDTH_Y as usize;
    let w_x: f32 = (n_x * SPIN_WIDTH_X) as f32;
    let w_y: f32 = (n_y * SPIN_WIDTH_Y) as f32;

    let mut lines: Vec<Line> = Vec::new();
    for i in 0..n_x {
        for j in 0..n_y {
            let x: f32 = (i * SPIN_WIDTH_X) as f32 / w_x;
            let y: f32 = (j * SPIN_WIDTH_Y) as f32 / w_y;
            let xp: f32 = 2.0*x*x + y;
            let yp: f32 = x*x;
            let vx: f32 = xp - x;
            let vy: f32 = yp - y;
            let theta = (vy/vx).atan();
            let r = 100.0 * (vx*vx + vy*vy).sqrt() / 2.0;
            let px: f32 = (i * SPIN_WIDTH_X) as f32 - 0.5 * w_x;
            let py: f32 = (j * SPIN_WIDTH_Y) as f32 - 0.5 * w_y;
            let start_x: f32 = (px - 0.5 * r * theta.cos()).min(0.5*w_x).max(-0.5*w_x);
            let start_y: f32 = (py - 0.5 * r * theta.sin()).min(0.5*w_y).max(-0.5*w_y);
            let end_x: f32 = (px + 0.5 * r * theta.cos()).min(0.5*w_x).max(-0.5*w_x);
            let end_y: f32 = (py + 0.5 * r * theta.sin()).min(0.5*w_y).max(-0.5*w_y);

            let dx = end_x - start_x;
            let dy = end_y - start_y;
            lines.push(Line{
                start: Point2::new(start_x, start_y),
                end: Point2::new(end_x, end_y),
                highlight_start: Point2::new(start_x, start_y),
                highlight_end: Point2::new(start_x + dx, start_y + dy),
                step: 0,
                n_highlight_steps: N_HIGHLIGHT_STEPS
            });
        }
    }

    Model {
        _window,
        lines,
        rng,
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    for line in _model.lines.iter_mut() {
        let sx = line.start.to_array()[0];
        let sy = line.start.to_array()[1];
        let ex = line.end.to_array()[0];
        let ey = line.end.to_array()[1];
        let hsx = line.highlight_start.to_array()[0];
        let hsy = line.highlight_start.to_array()[1];
        let hex = line.highlight_end.to_array()[0];
        let hey = line.highlight_end.to_array()[1];

        let dy = ey - sy;
        let dx = ex - sx;

        line.step += 1;
        if line.step >= line.n_highlight_steps {
            let hexp = sx + dx;
            let heyp = sy + dy;
            line.highlight_start = line.start;
            line.highlight_end = Point2::new(hexp, heyp);
            line.step = 0;
        } else {
            let hsxp = hsx + dx * (1.0 / line.n_highlight_steps as f32);
            let hsyp = hsy + dy * (1.0 / line.n_highlight_steps as f32);
            let hexp = hex + dx * (1.0 / line.n_highlight_steps as f32);
            let heyp = hey + dy * (1.0 / line.n_highlight_steps as f32);
            line.highlight_start = Point2::new(hsxp, hsyp);
            line.highlight_end = Point2::new(hexp, heyp);
        }
    }
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    //draw.background().color(BLACK);

    for line in _model.lines.iter() {
        // draw.line().color(WHITE).weight(5.0).points(line.start, line.end);
        draw.line().color(PINK).weight(1.0).points(line.highlight_start, line.highlight_end);
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
