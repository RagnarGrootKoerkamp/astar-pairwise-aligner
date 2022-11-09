use std::{sync::atomic::AtomicUsize, time::Duration};

use crate::{
    canvas::{Canvas, Color, HAlign, VAlign, BLACK},
    cli::ARGS,
    interaction::KeyboardAction,
};

use sdl2::{
    event::Event,
    keyboard::Keycode,
    rect::Rect,
    ttf::{Font, Sdl2TtfContext},
    video::Window,
    Sdl,
};
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

pub fn new_canvas(w: u32, h: u32) -> SdlCanvas {
    let video_subsystem = SDL_CONTEXT.with(|sdl| sdl.video().unwrap());
    video_subsystem.gl_attr().set_double_buffer(true);

    video_subsystem
        .window("Suffix array extension", w, h)
        //.borderless()
        .build()
        .unwrap()
        .into_canvas()
        .build()
        .unwrap()
}

impl Canvas for SdlCanvas {
    fn fill_background(&mut self, color: Color) {
        self.set_draw_color(color);
        self.fill_rect(Rect::new(
            0,
            0,
            self.output_size().unwrap().0,
            self.output_size().unwrap().1,
        ))
        .unwrap();
    }

    fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color) {
        self.set_draw_color(color);
        self.fill_rect(Rect::new(x, y, w, h)).unwrap();
    }

    fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color) {
        self.set_draw_color(color);
        self.draw_rect(Rect::new(x, y, w, h)).unwrap();
    }

    fn write_text(&mut self, x: i32, y: i32, ha: HAlign, va: VAlign, text: &str) {
        self.set_draw_color(BLACK);
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

    fn save(&mut self) {
        if let Some(mut path) = ARGS.save.clone() {
            static FRAME: AtomicUsize = AtomicUsize::new(0);

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

            std::fs::create_dir_all(&path).unwrap();
            let frame = FRAME.load(std::sync::atomic::Ordering::Acquire);
            // NOTE: We can not use zero-padded ints since ffmpeg can't handle it.
            path.push(format!("{frame}"));
            path.set_extension("bmp");
            surf.save_bmp(path).unwrap();
            FRAME.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        }
    }

    fn present(&mut self) {
        self.present()
    }
}

pub fn wait_for_key(timeout: Duration) -> KeyboardAction {
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
                        Keycode::Backspace | Keycode::Left => return KeyboardAction::Prev,
                        Keycode::P | Keycode::Return => return KeyboardAction::PausePlay,
                        Keycode::Plus | Keycode::Up | Keycode::F => return KeyboardAction::Faster,
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
