use nannou::prelude::*;
use rand::prelude::*;
use rand_pcg::Pcg64;

const SEED: u64 = 123456;
const WINDOW_WIDTH: usize = 1000;
const WINDOW_HEIGHT: usize = 1000;
const SQUARE_WIDTH: f32 = 500.0;
const SQUARE_HEIGHT: f32 = 500.0;
const MAX_BLOCKS: usize = 50;
const MAX_GATES: usize = 50;
const STEP_SIZE: f32 = 50.0;

struct Gate {
    is_closed: bool,
    is_open: bool,
    is_vertical: bool,
    blocks0: Vec<Rect>,
    blocks1: Vec<Rect>
}

struct Model {
    _window: window::Id,
    gates: Vec<Gate>,
    closing: bool,
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
    let rng = Pcg64::seed_from_u64(SEED);
    let window_width = WINDOW_WIDTH as f32;
    let window_height = WINDOW_HEIGHT as f32;

    let offset_x = (window_width - SQUARE_WIDTH) / 2.0;
    let offset_y = (window_height - SQUARE_HEIGHT) / 2.0;
    let x0: f32 = offset_x - 0.5*window_width;
    let y0: f32 = offset_y - 0.5*window_height;
    let x1: f32 = SQUARE_WIDTH + offset_x - 0.5*window_width;
    let y1: f32 = SQUARE_HEIGHT + offset_y - 0.5*window_height;
    let gates: Vec<Gate> = Vec::new();
    let closing: bool = true;

    Model {
        _window,
        gates,
        closing,
        x0,
        x1,
        y0,
        y1,
        rng
    }
}

fn create_gate(_model: &mut Model) -> Gate {
    // settings
    let is_vertical: bool = _model.rng.gen_bool(0.5);
    let window_height = _model.y1 - _model.y0;
    let window_width = _model.x1 - _model.x0;
    let n_blocks = _model.rng.gen_range(1..MAX_BLOCKS);

    // choose random mid-points
    let mut xs: Vec<f32> = Vec::new();
    let mut ys: Vec<f32> = Vec::new();
    let model_x0 = if is_vertical { _model.x0 } else { _model.x0 + 0.25*window_width };
    let model_x1 = if is_vertical { _model.x1 } else { _model.x1 - 0.25*window_width };
    let model_y0 = if is_vertical { _model.y0 + 0.25*window_width } else { _model.y0 };
    let model_y1 = if is_vertical { _model.y1 - 0.25*window_width } else { _model.y1 };
    for _ in 0..n_blocks {
        let x = _model.rng.gen_range(model_x0..model_x1);
        let y = _model.rng.gen_range(model_y0..model_y1);
        xs.push(x);
        ys.push(y);
    }

    // sort
    if is_vertical {
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    } else {
        ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }

    // generate blocks
    let mut blocks0: Vec<Rect> = Vec::new();
    let mut blocks1: Vec<Rect> = Vec::new();
    for i in 0..n_blocks {
        if is_vertical {
            let x0 = if i > 0 { (xs[i-1] + xs[i]) / 2.0 } else { _model.x0 };
            let x1 = if i < n_blocks - 1 { (xs[i] + xs[i+1]) / 2.0 } else { _model.x1 };
            let y = ys[i];

            let height0 = _model.y1 - y;
            let p00 = Point2::new(x0, _model.y1 + window_height);
            let p01 = Point2::new(x1, _model.y1 + window_height - height0);
            let block0: Rect = Rect::from_corners(p00, p01);
            blocks0.push(block0);

            let height1 = window_height - height0;
            let p10 = Point2::new(x0, _model.y0 - window_height + height1);
            let p11 = Point2::new(x1, _model.y0 - window_height);
            let block1: Rect = Rect::from_corners(p10, p11);
            blocks1.push(block1);
        } else {
            let x = xs[i];
            let y0 = if i > 0 { (ys[i-1] + ys[i]) / 2.0 } else { _model.y0 };
            let y1 = if i < n_blocks - 1 { (ys[i] + ys[i+1]) / 2.0 } else { _model.y1 };

            let width0 = x - _model.x0;
            let p00 = Point2::new(_model.x0 - window_width, y0);
            let p01 = Point2::new(_model.x0 - window_width + width0, y1);
            let block0: Rect = Rect::from_corners(p00, p01);
            blocks0.push(block0);

            let width1 = window_width - width0;
            let p10 = Point2::new(_model.x1 + window_width - width1, y0);
            let p11 = Point2::new(_model.x1 + window_width, y1);
            let block1: Rect = Rect::from_corners(p10, p11);
            blocks1.push(block1);
        }
    }

    let gate = Gate{
        is_closed: false,
        is_open: false,
        is_vertical: is_vertical,
        blocks0: blocks0,
        blocks1: blocks1
    };
    gate
}

fn evolve_gate(active_gate: &mut Gate, model_x0: f32, model_y1: f32, direction: bool) {
    let active_gate_blocks_iter = active_gate.blocks0.iter_mut().zip(active_gate.blocks1.iter_mut());
    let sign: f32 = if direction { 1.0 } else { -1.0 };
    let mut is_open: bool = true;
    for (block0, block1) in active_gate_blocks_iter {
        let mut block_is_open = false;
        // let mut x00 = block0.x.start;
        let mut x01 = block0.x.end;
        let mut y00 = block0.y.start;
        // let mut y01 = block0.y.end;
        let mut x10 = block1.x.start;
        // let mut x11 = block1.x.end;
        // let mut y10 = block1.y.start;
        let mut y11 = block1.y.end;

        let delta = STEP_SIZE;
        if active_gate.is_vertical {
            y00 -= sign * delta;
            // y01 -= delta;
            // y10 += delta;
            y11 += sign * delta;
            if y00 <= y11 {
                active_gate.is_closed = true;
            } else if !direction && y00 >= model_y1 {
                block_is_open = true;
            }
        } else {
            // x00 += delta;
            x01 += sign * delta;
            x10 -= sign * delta;
            // x11 -= delta;
            if x01 >= x10 {
                active_gate.is_closed = true;
            } else if !direction && x01 <= model_x0 {
                block_is_open = true;
            }
        }

        // block0.x.start = x00;
        block0.x.end = x01;
        block0.y.start = y00;
        // block0.y.end = y01;
        block1.x.start = x10;
        // block1.x.end = x11;
        // block1.y.start = y10;
        block1.y.end = y11;

        is_open = is_open && block_is_open;
    }
    active_gate.is_open = is_open;
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    let n_gates = _model.gates.len();
    println!("n_gates {}", n_gates);
    if n_gates > 0 {
        let mut active_gate = &mut _model.gates[n_gates-1];
        if _model.closing {
            if active_gate.is_closed {
                if n_gates >= MAX_GATES {
                    _model.closing = false;
                } else {
                    let gate = create_gate(_model);
                    _model.gates.push(gate);
                }
            } else {
                evolve_gate(&mut active_gate, _model.x0, _model.y1, true);
            }
        } else {
            if active_gate.is_open {
                if n_gates == 0 {
                    _model.closing = true;
                } else {
                    _model.gates.pop();
                }
            } else {
                evolve_gate(&mut active_gate, _model.x0, _model.y1, false);
            }
        }
    } else {
        _model.closing = true;
        let gate = create_gate(_model);
        _model.gates.push(gate);
    }
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    draw.rect().color(BLACK).stroke(PINK).stroke_weight(1.0).x(0.0).y(0.0).w(_model.x1 - _model.x0).h(_model.y1 - _model.y0);
    for gate in _model.gates.iter() {
        for block in gate.blocks0.iter().chain(gate.blocks1.iter()) {
            let (mut x, mut y, mut w, mut h) = block.x_y_w_h();
            let x0 = (x - w/2.0).min(_model.x1).max(_model.x0);
            let x1 = (x + w/2.0).min(_model.x1).max(_model.x0);
            let y0 = (y - h/2.0).min(_model.y1).max(_model.y0);
            let y1 = (y + h/2.0).min(_model.y1).max(_model.y0);
            x = (x0 + x1) / 2.0;
            y = (y0 + y1) / 2.0;
            w = x1 - x0;
            h = y1 - y0;
            draw.rect().color(BLACK).stroke(PINK).stroke_weight(1.0).x(x).y(y).w(w).h(h);
        }
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
