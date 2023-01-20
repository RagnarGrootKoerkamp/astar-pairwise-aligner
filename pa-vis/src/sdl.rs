use crate::canvas::*;
use lazy_static::lazy_static;
use pa_types::I;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    rect::{Point, Rect},
    ttf::{Font, Sdl2TtfContext},
    video::Window,
    Sdl,
};
use std::{path::Path, time::Duration};

use crate::canvas::{CPos, KeyboardAction};
pub type SdlCanvas = sdl2::render::Canvas<Window>;

lazy_static! {
    static ref TTF_CONTEXT: Sdl2TtfContext = sdl2::ttf::init().unwrap();
}

thread_local! {
    static SDL_CONTEXT: Sdl = sdl2::init().unwrap();
    static FONT: Font<'static, 'static> = TTF_CONTEXT
        .load_font("/usr/share/fonts/TTF/OpenSans-Regular.ttf", 24)
        .unwrap();
}

fn to_point(CPos(x, y): CPos) -> Point {
    Point::new(x as i32, y as i32)
}

struct SdlCanvasFactory;

impl CanvasFactory for SdlCanvasFactory {
    fn new(w: usize, h: usize, title: &str) -> Box<dyn Canvas> {
        let video_subsystem = SDL_CONTEXT.with(|sdl| sdl.video().unwrap());
        video_subsystem.gl_attr().set_double_buffer(true);

        Box::new(
            video_subsystem
                .window(title, w as u32, h as u32)
                //.borderless()
                .build()
                .unwrap()
                .into_canvas()
                .build()
                .unwrap(),
        )
    }
}

impl Canvas for SdlCanvas {
    fn fill_background(&mut self, color: Color) {
        self.set_draw_color(color);
        SdlCanvas::fill_rect(
            self,
            Rect::new(
                0,
                0,
                self.output_size().unwrap().0,
                self.output_size().unwrap().1,
            ),
        )
        .unwrap();
    }

    fn fill_rect(&mut self, CPos(x, y): CPos, w: I, h: I, color: Color) {
        self.set_draw_color(color);
        SdlCanvas::fill_rect(self, Rect::new(x as i32, y as i32, w as u32, h as u32)).unwrap();
    }

    fn fill_rects(&mut self, rects: &[(CPos, I, I)], color: Color) {
        self.set_draw_color(color);
        let rects: Vec<_> = rects
            .iter()
            .map(|&(CPos(x, y), w, h)| Rect::new(x as i32, y as i32, w as u32, h as u32))
            .collect();
        SdlCanvas::fill_rects(self, &rects).unwrap();
    }

    fn draw_rect(&mut self, CPos(x, y): CPos, w: I, h: I, color: Color) {
        self.set_draw_color(color);
        SdlCanvas::draw_rect(self, Rect::new(x as i32, y as i32, w as u32, h as u32)).unwrap();
    }

    fn draw_point(&mut self, p: CPos, color: Color) {
        self.set_draw_color(color);
        SdlCanvas::draw_point(self, to_point(p)).unwrap();
    }

    fn draw_line(&mut self, p: CPos, q: CPos, color: Color) {
        self.set_draw_color(color);
        SdlCanvas::draw_line(self, to_point(p), to_point(q)).unwrap();
    }

    fn write_text(&mut self, CPos(x, y): CPos, ha: HAlign, va: VAlign, text: &str, color: Color) {
        self.set_draw_color(color);
        let surface = FONT.with(|front| front.render(text).blended(self.draw_color()).unwrap());

        let w = surface.width();
        let h = surface.height();
        let x = match ha {
            HAlign::Left => x,
            HAlign::Center => x - w as i32 / 2,
            HAlign::Right => x - w as i32,
        };
        let y = match va {
            VAlign::Top => y,
            VAlign::Center => y - h as i32 / 2,
            VAlign::Bottom => y - h as i32,
        };
        let texture_creator = self.texture_creator();
        self.copy(
            &surface.as_texture(&texture_creator).unwrap(),
            None,
            Some(Rect::new(x, y, w, h)),
        )
        .unwrap();
    }

    fn save(&mut self, path: &Path) {
        let pixel_format = self.default_pixel_format();
        let mut pixels = self.read_pixels(self.viewport(), pixel_format).unwrap();
        let (width, height) = self.output_size().unwrap();
        let pitch = pixel_format.byte_size_of_pixels(width as usize);
        let surf = sdl2::surface::Surface::from_data(
            pixels.as_mut_slice(),
            width,
            height,
            pitch as u32,
            pixel_format,
        )
        .unwrap();

        std::fs::create_dir_all(&path.parent().unwrap()).unwrap();
        surf.save_bmp(path).unwrap();
    }

    fn present(&mut self) {
        self.present()
    }

    fn wait(&mut self, timeout: Duration) -> KeyboardAction {
        let step = Duration::from_secs_f32(0.01);
        SDL_CONTEXT.with(|sdl| {
            for _ in 0..=timeout.as_nanos() / step.as_nanos() {
                for event in sdl.event_pump().unwrap().poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::X),
                            ..
                        } => return KeyboardAction::Exit,
                        Event::KeyDown {
                            keycode: Some(key), ..
                        } => match key {
                            Keycode::Space | Keycode::Right => return KeyboardAction::Next,
                            //Keycode::Backspace | Keycode::Left => return KeyboardAction::Prev,
                            Keycode::P | Keycode::Return => return KeyboardAction::PausePlay,
                            Keycode::Plus | Keycode::Up | Keycode::F => {
                                return KeyboardAction::Faster
                            }
                            Keycode::Minus | Keycode::Down | Keycode::S => {
                                return KeyboardAction::Slower
                            }
                            Keycode::Escape | Keycode::Q => return KeyboardAction::ToEnd,
                            _ => {}
                        },
                        _ => {}
                    }
                }
                ::std::thread::sleep(step);
            }
            return KeyboardAction::None;
        })
    }
}
