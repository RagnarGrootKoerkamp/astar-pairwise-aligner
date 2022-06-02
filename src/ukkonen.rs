use num_traits::abs;
use sdl2::pixels::Color;

use crate::{astar::Config, drawing::display2, prelude::*};

const path_loop: usize = 60;

fn ukkonen_track_path<'a>(
    A: &Vec<Vec<usize>>,
    s1: &'a Sequence,
    s2: &'a Sequence,
    len: usize,
    p: usize,
    t: usize,
) -> Vec<Pos> {
    let mut path = vec![];
    let mut i = A.len() - 1;
    let mut j = t - p;
    let convert = |x: usize, y: usize| -> Pos { Pos((x + y - p) as u32, x as u32) };
    let mut f = false;
    let mut save_pos = |i, j| -> () {
        path.push(convert(i, j));
    };
    save_pos(i, j);
    loop {
        if i > 0 {
            if s1[i - 1] == s2[i + j - p - 1] {
                if f {
                    save_pos(i, j);
                    f = false;
                }
                i -= 1;
            } else {
                save_pos(i, j);
                f = true;
                if A[i][j] == A[i - 1][j] + 1 {
                    i -= 1;
                } else if j < t && A[i][j] == A[i - 1][j + 1] + 1 {
                    i -= 1;
                    j += 1;
                } else {
                    j -= 1;
                }
            }
        } else {
            save_pos(i, j);
            j -= 1;
        }
        if i == 0 && j == p {
            path.reverse();
            println!("Path size is {}", path.len());
            return path;
        }
    }
}

pub fn ukkonen<'a>(mut s1: &'a Sequence, mut s2: &'a Sequence, d: usize) -> usize {
    let mut len1 = s1.len();
    let mut len2 = s2.len();
    if len1 > len2 {
        (s1, s2) = (s2, s1);
        (len1, len2) = (len2, len1);
    }
    let p: usize = ((d - abs(len2 as i32 - len1 as i32) as usize) / 2) as usize;
    let t: usize = (len2 - len1) + 2 * p;
    if t < 2 {
        return d + 1;
    }
    let mut A = vec![vec![0usize; (t + 1) as usize]; len1 + 1];
    let mut j;

    for i in p..=t {
        A[0][i] = i - p;
    }

    for i in 0..=p {
        A[i][p - i] = i;
    }

    for i in 1..=len1 {
        if i > p {
            if s1[i - 1] == s2[i - p - 1] {
                A[i][0] = A[i - 1][0];
            } else {
                A[i][0] = min(A[i - 1][0], A[i - 1][1]) + 1;
            }
        } else if i == p {
            A[i][0] = A[i - 1][1] + 1;
        }

        for jj in 1..t {
            if i + jj < p + 1 {
                continue;
            }

            if let Some(&c2) = s2.get(i + jj - p - 1) {
                if s1[i - 1] == c2 {
                    A[i][jj] = A[i - 1][jj];
                } else {
                    A[i][jj] = min(min(A[i - 1][jj], A[i - 1][jj + 1]), A[i][jj - 1]) + 1;
                }
            }
        }
        j = t;
        if i + j >= p + 1 && i + j - p - 1 < len2 {
            if s1[i - 1] == s2[i + j - p - 1] {
                A[i][j] = A[i - 1][j];
            } else {
                A[i][j] = min(A[i - 1][j], A[i][j - 1]) + 1;
            }
        } else {
            A[i][j] = min(A[i - 1][j], A[i][j - 1]) + 1;
        }
    }

    A[len1][t - p]
}

pub fn ukkonen_vis<'a>(
    mut s1: &'a Sequence,
    mut s2: &'a Sequence,
    d: usize,
    config: &Config,
    mut file_number: usize,
    mut is_playing: bool,
    mut skip: usize,
    prev: &mut Vec<Vec<Pos>>,
) -> (usize, usize, bool, usize) {
    // (edit distance, file_number, is_playing, skip)
    let mut explored_states: Vec<Pos> = vec![];
    let mut expanded_states: Vec<Pos> = vec![Pos(0, 0); 1];
    //expanded_states.push(Pos(0, 0));

    let mut len1 = s1.len();
    let mut len2 = s2.len();
    if len1 > len2 {
        (s1, s2) = (s2, s1);
        (len1, len2) = (len2, len1);
    }
    let mut p: usize = ((d - abs(len2 as i32 - len1 as i32) as usize) / 2) as usize;
    let mut t: usize = (len2 - len1) + 2 * p;
    if file_number == 0 && len1 == len2 {
        // DELETE IT
        t = 0;
        p = 0;
    } else if t < 2 {
        return (d + 1, file_number, is_playing, skip);
    }

    println!("p is {p}\nt is {t}");

    let low = Pos(0, 0);
    let high = Pos((len2) as u32, (len1) as u32);
    println!("len1 is {len1}\nlen2 is {len2}");
    const CELL_SIZE: u32 = 8;
    let mut sdl_context = sdl2::init().unwrap();
    let canvas_size_cells = Pos(high.0 - low.0 + 1, high.1 - low.1 + 1);
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.gl_attr().set_double_buffer(true);
    let window = if config.drawing {
        Some(
            video_subsystem
                .window(
                    "A*PA",
                    canvas_size_cells.0 as u32 * CELL_SIZE * SCALE,
                    (canvas_size_cells.1 as u32) * CELL_SIZE * SCALE + v_offset * SCALE,
                )
                .borderless()
                .build()
                .unwrap(),
        )
    } else {
        None
    };
    let ref mut canvas = window.map(|w| w.into_canvas().build().unwrap());
    println!(
        "Window width is {}\nHeight is {}\n",
        canvas_size_cells.0 as u32 * CELL_SIZE * SCALE,
        (canvas_size_cells.1 as u32) * CELL_SIZE * SCALE + v_offset * SCALE
    );

    let mut A = vec![vec![0usize; (t + 1) as usize]; len1 + 1];
    let mut j;

    for i in p..=t {
        A[0][i] = i - p;
        explored_states.push(Pos((i - p) as u32, 0));
    }

    for i in 0..=p {
        A[i][p - i] = i;
        explored_states.push(Pos(0, i as u32));
    }

    for i in 1..=len1 {
        if i > p {
            if s1[i - 1] == s2[i - p - 1] {
                A[i][0] = A[i - 1][0];
            } else {
                if t > 0 {
                    //DELETE THIS if
                    A[i][0] = min(A[i - 1][0], A[i - 1][1]) + 1;
                } else {
                    A[i][0] = A[i - 1][0] + 1;
                }
            }
        } else if i == p {
            A[i][0] = A[i - 1][1] + 1;
        }

        if i > p {
            if let Some(canvas) = canvas {
                expanded_states[0] = Pos((i - p) as u32, (i) as u32);
                let path: Option<(u32, Vec<Pos>)> = None; //TODO: make path
                let tmp3: Option<(u32, Vec<Pos>)>;
                let mut tmp4: Vec<Pos> = vec![Pos(0, 0)];
                if path != None {
                    tmp3 = path.clone();
                    tmp4 = tmp3.unwrap_or_default().1;
                }

                (is_playing, file_number, skip) = display2(
                    high,
                    Some(&explored_states),
                    Some(&expanded_states),
                    Some(&prev),
                    if path == None { None } else { Some(&tmp4) },
                    None,
                    canvas_size_cells,
                    canvas,
                    &mut sdl_context,
                    is_playing,
                    &config,
                    file_number,
                    skip,
                    Color::RGBA(0, 0, 255, 50),
                );
                explored_states.push(expanded_states[0]);
            }
        }

        for jj in 1..t {
            if i + jj < p + 1 {
                continue;
            }

            if let Some(&c2) = s2.get(i + jj - p - 1) {
                if s1[i - 1] == c2 {
                    A[i][jj] = A[i - 1][jj];
                } else {
                    A[i][jj] = min(min(A[i - 1][jj], A[i - 1][jj + 1]), A[i][jj - 1]) + 1;
                }
            }

            if let Some(canvas) = canvas {
                expanded_states[0] = Pos((i + jj - p) as u32, (i) as u32);
                let path: Option<(u32, Vec<Pos>)> = None; //TODO: make path
                let tmp3: Option<(u32, Vec<Pos>)>;
                let mut tmp4: Vec<Pos> = vec![Pos(0, 0)];
                if path != None {
                    tmp3 = path.clone();
                    tmp4 = tmp3.unwrap_or_default().1;
                }

                (is_playing, file_number, skip) = display2(
                    high,
                    Some(&explored_states),
                    Some(&expanded_states),
                    Some(&prev),
                    if path == None { None } else { Some(&tmp4) },
                    None,
                    canvas_size_cells,
                    canvas,
                    &mut sdl_context,
                    is_playing,
                    &config,
                    file_number,
                    skip,
                    Color::RGBA(0, 0, 255, 50),
                );
                explored_states.push(expanded_states[0]);
            }
        }
        j = t;
        if i + j >= p + 1 && i + j - p - 1 < len2 {
            if s1[i - 1] == s2[i + j - p - 1] {
                A[i][j] = A[i - 1][j];
            } else {
                if j > 0 {
                    //DELETE THIS IF
                    A[i][j] = min(A[i - 1][j], A[i][j - 1]) + 1;
                } else {
                    A[i][j] = A[i - 1][j] + 1;
                }
            }
        } else {
            A[i][j] = min(A[i - 1][j], A[i][j - 1]) + 1;
        }
        if let Some(canvas) = canvas {
            expanded_states[0] = Pos((i + j - p) as u32, (i) as u32);
            let path: Option<(u32, Vec<Pos>)> = None; //TODO: make path
            let tmp3: Option<(u32, Vec<Pos>)>;
            let mut tmp4: Vec<Pos> = vec![Pos(0, 0)];
            if path != None {
                tmp3 = path.clone();
                tmp4 = tmp3.unwrap_or_default().1;
            }

            (is_playing, file_number, skip) = display2(
                high,
                Some(&explored_states),
                Some(&expanded_states),
                Some(&prev),
                if path == None { None } else { Some(&tmp4) },
                None,
                canvas_size_cells,
                canvas,
                &mut sdl_context,
                is_playing,
                &config,
                file_number,
                skip,
                Color::RGBA(0, 0, 255, 50),
            );
            explored_states.push(expanded_states[0]);
        }
    }

    if let Some(canvas) = canvas {
        let tmp = A[len1][t - p];
        let path = ukkonen_track_path(&A, s1, s2, tmp, p, t);
        for tmp3 in 0..path_loop {
            (is_playing, file_number, skip) = display2(
                high,
                Some(&explored_states),
                None,
                None,
                Some(&path),
                None,
                canvas_size_cells,
                canvas,
                &mut sdl_context,
                is_playing,
                &config,
                file_number,
                skip,
                Color::RGBA(0, 0, 255, 50),
            );
        }
    }
    prev.push(explored_states);
    (A[len1][t - p], file_number, is_playing, skip)
}

pub fn ukkonen2<'a>(
    mut s1: &'a Sequence,
    mut s2: &'a Sequence,
    d: usize,
    explored: &mut Vec<Pos>,
) -> (usize, Vec<Pos>) {
    let mut len1 = s1.len();
    let mut len2 = s2.len();
    if len1 > len2 {
        (s1, s2) = (s2, s1);
        (len1, len2) = (len2, len1);
    }
    let p: usize = ((d - abs(len2 as i32 - len1 as i32) as usize) / 2) as usize;
    let t: usize = (len2 - len1) + 2 * p;
    if t < 2 {
        return (d + 1, vec![]);
    }
    let mut A = vec![vec![0usize; (t + 1) as usize]; len1 + 1];
    let mut j;

    for i in p..=t {
        A[0][i] = i - p;
        explored.push(Pos((i - p) as u32, 0));
        //explored.push(Pos(0, i as u32));
    }

    for i in 0..=p {
        A[i][p - i] = i;
        explored.push(Pos(0, i as u32));
        //explored.push(Pos(i as u32, (p - i) as u32));
    }

    for i in 1..=len1 {
        if i > p {
            if s1[i - 1] == s2[i - p - 1] {
                A[i][0] = A[i - 1][0];
            } else {
                A[i][0] = min(A[i - 1][0], A[i - 1][1]) + 1;
            }
            explored.push(Pos((i - p) as u32, (i) as u32));
        } else if i == p {
            A[i][0] = A[i - 1][1] + 1;
            explored.push(Pos((i - p) as u32, (i) as u32));
        }

        for jj in 1..t {
            if i + jj < p + 1 {
                continue;
            }

            if let Some(&c2) = s2.get(i + jj - p - 1) {
                if s1[i - 1] == c2 {
                    A[i][jj] = A[i - 1][jj];
                } else {
                    A[i][jj] = min(min(A[i - 1][jj], A[i - 1][jj + 1]), A[i][jj - 1]) + 1;
                }
                explored.push(Pos((i + jj - p) as u32, (i) as u32));
            }
        }
        j = t;
        if i + j >= p + 1 && i + j - p - 1 < len2 {
            if s1[i - 1] == s2[i + j - p - 1] {
                A[i][j] = A[i - 1][j];
            } else {
                A[i][j] = min(A[i - 1][j], A[i][j - 1]) + 1;
            }
        } else {
            A[i][j] = min(A[i - 1][j], A[i][j - 1]) + 1;
        }
        explored.push(Pos((i + j - p) as u32, (i) as u32));
    }

    let tmp = A[len1][t - p];
    let path = ukkonen_track_path(&A, s1, s2, tmp, p, t);
    (A[len1][t - p], path)
}
