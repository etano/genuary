use nannou::prelude::*;
use rand::prelude::*;
use rand_pcg::Pcg64;

const SEED: u64 = 123456;
const WINDOW_WIDTH: usize = 800;
const WINDOW_HEIGHT: usize = 80;
const SQUARE_WIDTH: f32 = 800.0;
const SQUARE_HEIGHT: f32 = 80.0;
const GRID_WIDTH: f32 = 10.0;
const GRID_HEIGHT: f32 = 10.0;

struct Shape {
    points: Vec<Vec<usize>>,
    is_moving: bool
}

struct Model {
    _window: window::Id,
    shapes: Vec<Shape>,
    is_complete: bool,
    grid: Vec<Vec<bool>>,
    grid_width: f32,
    grid_height: f32,
    n_x: usize,
    n_y: usize,
    x0: f32,
    // x1: f32,
    y0: f32,
    // y1: f32,
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
    // let x1: f32 = SQUARE_WIDTH + offset_x - 0.5*window_width;
    // let y1: f32 = SQUARE_HEIGHT + offset_y - 0.5*window_height;
    let shapes: Vec<Shape> = Vec::new();
    let is_complete: bool = false;
    let mut grid: Vec<Vec<bool>> = Vec::new();
    let grid_width: f32 = GRID_WIDTH;
    let grid_height: f32 = GRID_HEIGHT;
    let n_x: usize = (SQUARE_WIDTH / GRID_WIDTH) as usize;
    let n_y: usize = (SQUARE_HEIGHT / GRID_HEIGHT) as usize;
    for _ in 0..n_y {
        let mut v: Vec<bool> = Vec::new();
        for _ in 0..n_x {
            v.push(false);
        }
        grid.push(v);
    }

    Model {
        _window,
        shapes,
        is_complete,
        grid,
        grid_width,
        grid_height,
        n_x,
        n_y,
        x0,
        // x1,
        y0,
        // y1,
        rng
    }
}

fn get_max_i(row: &Vec<bool>) -> usize {
    let max_i_iter = row.iter().position(|&x| x);
    let max_i: usize = if max_i_iter != None { max_i_iter.unwrap() } else { row.len() };
    max_i
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    // evolve shapes
    for shape in _model.shapes.iter_mut() {
        if !shape.is_moving {
            continue;
        }
        let mut new_points: Vec<Vec<usize>> = Vec::new();

        // disappear shape
        for point in shape.points.iter() {
            let i: usize = point[0];
            let j: usize = point[1];
            _model.grid[j][i] = false;
        }

        // get max possible values
        let mut max_is: Vec<usize> = Vec::new();
        for j in 0.._model.n_y {
            max_is.push(get_max_i(&_model.grid[j]));
        }
        let mut shape_min_i: usize = _model.n_x - 1;
        let mut max_step_size: usize = _model.n_x - 1;
        for point in shape.points.iter() {
            let max_i = max_is[point[1]];
            max_step_size = max_step_size.min(max_i - point[0] - 1).min(_model.n_x - point[0] - 1).max(0);
            shape_min_i = shape_min_i.min(point[0]);
        }

        // move shape
        for point in shape.points.iter() {
            let mut i: usize = point[0];
            let j: usize = point[1];
            let max_i = max_is[j];
            if max_i == 0 || i >= max_i - 1 {
                shape.is_moving = false;
            } else {
                let step_size = (1 + (20.0 * (shape_min_i as f32 / _model.n_x as f32)) as usize).min(max_step_size);
                i += step_size;
                i = i.min(max_i - 1);
            }
            new_points.push(vec![i, j]);
        }
        if shape.is_moving {
            shape.points = new_points;
        }

        // reappear shape
        for point in shape.points.iter() {
            let i: usize = point[0];
            let j: usize = point[1];
            _model.grid[j][i] = true;
        }
    }

    // check if complete and reset
    for j in 0.._model.n_y {
        _model.is_complete = _model.is_complete || get_max_i(&_model.grid[j]) == 0;
    }
    if _model.is_complete {
        _model.shapes = Vec::new();
        for j in 0.._model.n_y {
            for i in 0.._model.n_x {
                _model.grid[j][i] = false;
            }
        }
        _model.is_complete = false;
    }

    // add shapes
    if _model.shapes.iter().filter(|&s| s.is_moving).count() == 0 {
        let mut points: Vec<Vec<i8>> = Vec::new();
        let mut j: usize = _model.rng.gen_range(0.._model.n_y);
        while get_max_i(&_model.grid[j]) == 0 {
            j = _model.rng.gen_range(0.._model.n_y);
        }
        points.push(vec![0, j as i8]);
        while points.len() < 4 {
            let ref last_point = points[points.len() - 1];
            let mut v = vec![last_point[0], last_point[1]];
            let x = _model.rng.gen_range(0..=1);
            if _model.rng.gen_range(0..=1) == 0 {
                v[x] -= 1;
            } else {
                v[x] += 1;
            }
            points.push(v);
            points.sort_unstable();
            points.dedup();
        }
        let mut max_i = 0;
        let mut min_i = 0;
        let mut max_j = 0;
        let mut min_j = 0;
        for point in points.iter() {
            max_i = max_i.max(point[0]);
            min_i = min_i.min(point[0]);
            max_j = max_j.max(point[1]);
            min_j = min_j.min(point[1]);
        }
        if min_i < 0 {
            let diff = min_i - 0;
            for point in points.iter_mut() {
                point[0] -= diff;
            }
        }
        if max_i > _model.n_x as i8 - 1 {
            let diff = max_i - (_model.n_x as i8 - 1);
            for point in points.iter_mut() {
                point[0] -= diff;
            }
        }
        if min_j < 0 {
            let diff = min_j - 0;
            for point in points.iter_mut() {
                point[1] -= diff;
            }
        }
        if max_j > _model.n_y as i8 - 1 {
            let diff = max_j - (_model.n_y as i8 - 1);
            for point in points.iter_mut() {
                point[1] -= diff;
            }
        }

        let mut new_points: Vec<Vec<usize>> = Vec::new();
        for point in points.iter() {
            new_points.push(vec![point[0] as usize, point[1] as usize]);
        }
        let shape = Shape{
            points: new_points,
            is_moving: true
        };
        _model.shapes.push(shape);
    }
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    for i in 0.._model.n_x {
        for j in 0.._model.n_y {
            if _model.grid[j][i] {
                draw.rect()
                    .color(PINK)
                    .stroke(BLACK)
                    .stroke_weight(1.0)
                    .x(_model.x0 + (i as f32 + 0.5) * _model.grid_width)
                    .y(_model.y0 + (j as f32 + 0.5) * _model.grid_height)
                    .w(_model.grid_width)
                    .h(_model.grid_height);
            }
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
