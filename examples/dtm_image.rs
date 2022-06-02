use pairwise_aligner::{
    diagonal_transition::diagonal_transition2,
    drawing::save_canvas,
    prelude::{setup, v_offset, Pos},
};
use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
};

fn main() {
    let n = 500;
    let e = 0.2;

    let m = 0;
    let k = 3;

    let (ref a, ref b, ref alphabet, stats) = setup(n, e);

    let _target = Pos::from_length(&a, &b);
    let mut explored = vec![];
    let mut path: Vec<Pos> = vec![];
    let mut r;

    (r, path) = diagonal_transition2(a, b, &mut explored);

    println!("DTM says that edit distance is {}", r);

    let low = Pos(0, 0);
    let high = _target;
    let width = high.0 - low.0 + 1;
    let height = high.1 - low.1 + 1;

    let mut sdl_context = sdl2::init().unwrap();
    let canvas_size_cells = Pos(high.0 - low.0 + 1, high.1 - low.1 + 1);
    let video_subsystem = sdl_context.video().unwrap();
    const CELL_SIZE: u32 = 1;
    const SCALE: u32 = 1;
    video_subsystem.gl_attr().set_double_buffer(true);
    let window = video_subsystem
        .window(
            "A*PA",
            canvas_size_cells.0 as u32 * CELL_SIZE * SCALE,
            (canvas_size_cells.1 as u32) * CELL_SIZE * SCALE + v_offset * SCALE,
        )
        .borderless()
        .build()
        .unwrap();
    let ref mut canvas = window.into_canvas().build().unwrap();

    /*let gray_bg = 0.97; //BG COLOR
    canvas.set_draw_color(Color::RGB(
        (gray_bg * 255.) as u8,
        (gray_bg * 255.) as u8,
        (gray_bg * 255.) as u8,
    ));
    canvas.fill_rect(Rect::new(0, 0, width, height));*/

    canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
    canvas.fill_rect(Rect::new(0, 0, width, height));

    let grad = colorgrad::turbo();
    let min_step = 0;
    let max_steps = explored.len() - 1;
    explored.reverse();
    for (i, pos) in explored.iter().enumerate() {
        let val = (max_steps - i - min_step) as f64 / (max_steps - min_step) as f64;
        let clr = grad.at(0.25 + (val * 0.65)).rgba_u8();
        canvas.set_draw_color(Color::RGB(clr.0, clr.1, clr.2));
        canvas.draw_point(Point::new(pos.1 as i32, pos.0 as i32));
    }

    let draw_thick_line_diag =
        |canvas: &mut Canvas<Window>, mut from: Point, mut to: Point, mut width: usize| {
            to *= SCALE as i32;
            from *= SCALE as i32;
            width *= SCALE as usize;
            canvas.draw_line(from, to).unwrap();
            for mut w in 1..width as i32 {
                if w % 2 == 1 {
                    w = (w + 1) / 2;
                    canvas
                        .draw_line(
                            Point::new(from.x + w, from.y - w + 1),
                            Point::new(to.x + w - 1, to.y - w),
                        )
                        .unwrap();
                    canvas
                        .draw_line(
                            Point::new(from.x - w, from.y + w - 1),
                            Point::new(to.x - w + 1, to.y + w),
                        )
                        .unwrap();
                    canvas
                        .draw_line(
                            Point::new(from.x + w - 1, from.y - w),
                            Point::new(to.x + w, to.y - w + 1),
                        )
                        .unwrap();
                    canvas
                        .draw_line(
                            Point::new(from.x - w + 1, from.y + w),
                            Point::new(to.x - w, to.y + w - 1),
                        )
                        .unwrap();
                } else {
                    w /= 2;
                    canvas
                        .draw_line(
                            Point::new(from.x + w, from.y - w),
                            Point::new(to.x + w, to.y - w),
                        )
                        .unwrap();
                    canvas
                        .draw_line(
                            Point::new(from.x - w, from.y + w),
                            Point::new(to.x - w, to.y + w),
                        )
                        .unwrap();
                }
            }
        };

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    let mut prev = Pos(0, 0);
    for p in path {
        draw_thick_line_diag(
            canvas,
            Point::new(prev.0 as i32, prev.1 as i32),
            Point::new(p.0 as i32, p.1 as i32),
            1,
        );
        prev = p;
    }

    save_canvas(&canvas, "evals/astar-visualization/WFA_transp", 0);
}
