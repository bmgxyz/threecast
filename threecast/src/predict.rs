use crate::parse::GridData;

#[allow(clippy::ptr_arg)]
fn shift(input: &GridData, dx: i32, dy: i32) -> GridData {
    let y_n = input.len() as i32;
    let x_n = input[0].len() as i32;
    let mut shifted: GridData = input.clone();
    for (y_t1, y_t2) in (0.max(-dy)..y_n.min(y_n - dy)).zip(0.max(dy)..y_n.min(y_n + dy)) {
        shifted.push(Vec::new());
        for (x_t1, x_t2) in (0.max(-dx)..x_n.min(x_n - dx)).zip(0.max(dx)..x_n.min(x_n + dx)) {
            shifted[y_t2 as usize][x_t2 as usize].1 = input[y_t1 as usize][x_t1 as usize].1;
        }
    }
    shifted
}

#[allow(clippy::ptr_arg)]
fn compute_mse_for_offset(t1: &GridData, t2: &GridData, dx: i32, dy: i32) -> f32 {
    let y_n = t1.len() as i32;
    let x_n = t1[0].len() as i32;
    let mut errors = 0.;
    // the fancy loop bounds here are due to Logan Boyd (@loboyd)
    // (so if you don't understand them, ask him)
    for (y_t1, y_t2) in (0.max(-dy)..y_n.min(y_n - dy)).zip(0.max(dy)..y_n.min(y_n + dy)) {
        for (x_t1, x_t2) in (0.max(-dx)..x_n.min(x_n - dx)).zip(0.max(dx)..x_n.min(x_n + dx)) {
            errors +=
                (t1[y_t1 as usize][x_t1 as usize].1 - t2[y_t2 as usize][x_t2 as usize].1).powi(2);
        }
    }
    let n = (t1.len() * t1[0].len()) as f32;
    errors / n
}

#[allow(clippy::ptr_arg)]
fn find_best_offset(t1: &GridData, t2: &GridData) -> (i32, i32) {
    let n = t1.len() as i32;
    // TODO: compute r in terms of physical pixel size and maximum reasonable storm speed
    let r = match n {
        n if n <= 0 => unreachable!(),
        n if n < 50 => n / 2,
        n if n >= 50 => n / 20,
        _ => unreachable!(),
    };
    let (x_0, x_n, y_0, y_n) = (-r, r, -r, r);
    let mut best_offset = (0, 0);
    let mut best_mse = f32::MAX;
    for dy in y_0..y_n {
        for dx in x_0..x_n {
            let error = compute_mse_for_offset(t1, t2, dx, dy);
            if error < best_mse {
                best_mse = error;
                best_offset = (dy, dx);
            }
        }
    }
    best_offset
}

/// Given two input grids of the same dimensions separated by `delta_t_now`
/// seconds, predict the precipitation from t = 0 to t = 60 minutes in
/// five-minute increments. `delta_t_now` is the number of seconds between the
/// second input grid and t = 0.
///
/// This function and its supporting logic depend on the simplest possible
/// solution that I could think of, which I call **DumbFlow**. The solution
/// assumes that there is some vector that describes the motion of all
/// precipitation between the first and second input grids. This ignores changes
/// in intensity or relative position. The solution estimates this offset vector
/// by trying more or less all possibilities and choosing the one with the
/// lowest mean-squared error. Then, it simply runs time forward by assuming
/// that the offset vector holds for all future values of t.
pub fn predict_two(input: [&GridData; 2], delta_t_image: u16, delta_t_now: u16) -> [GridData; 13] {
    let offset = find_best_offset(input[0], input[1]);
    let offset_per_second = (
        offset.0 as f32 / delta_t_image as f32,
        offset.1 as f32 / delta_t_image as f32,
    );
    [
        shift(
            input[0],
            (offset_per_second.1 * delta_t_now as f32) as i32,
            (offset_per_second.0 * delta_t_now as f32) as i32,
        ),
        shift(
            input[0],
            (offset_per_second.1 * (delta_t_now + 5 * 60) as f32) as i32,
            (offset_per_second.0 * (delta_t_now + 5 * 60) as f32) as i32,
        ),
        shift(
            input[0],
            (offset_per_second.1 * (delta_t_now + 10 * 60) as f32) as i32,
            (offset_per_second.0 * (delta_t_now + 10 * 60) as f32) as i32,
        ),
        shift(
            input[0],
            (offset_per_second.1 * (delta_t_now + 15 * 60) as f32) as i32,
            (offset_per_second.0 * (delta_t_now + 15 * 60) as f32) as i32,
        ),
        shift(
            input[0],
            (offset_per_second.1 * (delta_t_now + 20 * 60) as f32) as i32,
            (offset_per_second.0 * (delta_t_now + 20 * 60) as f32) as i32,
        ),
        shift(
            input[0],
            (offset_per_second.1 * (delta_t_now + 25 * 60) as f32) as i32,
            (offset_per_second.0 * (delta_t_now + 25 * 60) as f32) as i32,
        ),
        shift(
            input[0],
            (offset_per_second.1 * (delta_t_now + 30 * 60) as f32) as i32,
            (offset_per_second.0 * (delta_t_now + 30 * 60) as f32) as i32,
        ),
        shift(
            input[0],
            (offset_per_second.1 * (delta_t_now + 35 * 60) as f32) as i32,
            (offset_per_second.0 * (delta_t_now + 35 * 60) as f32) as i32,
        ),
        shift(
            input[0],
            (offset_per_second.1 * (delta_t_now + 40 * 60) as f32) as i32,
            (offset_per_second.0 * (delta_t_now + 40 * 60) as f32) as i32,
        ),
        shift(
            input[0],
            (offset_per_second.1 * (delta_t_now + 45 * 60) as f32) as i32,
            (offset_per_second.0 * (delta_t_now + 45 * 60) as f32) as i32,
        ),
        shift(
            input[0],
            (offset_per_second.1 * (delta_t_now + 50 * 60) as f32) as i32,
            (offset_per_second.0 * (delta_t_now + 50 * 60) as f32) as i32,
        ),
        shift(
            input[0],
            (offset_per_second.1 * (delta_t_now + 55 * 60) as f32) as i32,
            (offset_per_second.0 * (delta_t_now + 55 * 60) as f32) as i32,
        ),
        shift(
            input[0],
            (offset_per_second.1 * (delta_t_now + 60 * 60) as f32) as i32,
            (offset_per_second.0 * (delta_t_now + 60 * 60) as f32) as i32,
        ),
    ]
}

#[test]
fn find_best_offset_simple() {
    let t1: GridData = vec![
        vec![([0, 0], 1.), ([0, 0], 1.), ([0, 0], 1.), ([0, 0], 1.)],
        vec![([0, 0], 1.), ([0, 0], 0.), ([0, 0], 0.), ([0, 0], 0.)],
        vec![([0, 0], 1.), ([0, 0], 0.), ([0, 0], 0.), ([0, 0], 0.)],
        vec![([0, 0], 1.), ([0, 0], 0.), ([0, 0], 0.), ([0, 0], 0.)],
    ];
    let t2: GridData = vec![
        vec![([0, 0], 0.), ([0, 0], 0.), ([0, 0], 0.), ([0, 0], 0.)],
        vec![([0, 0], 0.), ([0, 0], 1.), ([0, 0], 1.), ([0, 0], 1.)],
        vec![([0, 0], 0.), ([0, 0], 1.), ([0, 0], 0.), ([0, 0], 0.)],
        vec![([0, 0], 0.), ([0, 0], 1.), ([0, 0], 0.), ([0, 0], 0.)],
    ];
    assert_eq!(find_best_offset(&t1, &t2), (1, 1));
}
