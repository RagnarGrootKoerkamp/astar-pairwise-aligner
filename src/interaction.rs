use std::time::Duration;

pub struct Interaction {
    len: usize,
    idx: usize,
    forward: bool,
    spf: Duration,
    playing: bool,
}

#[derive(PartialEq, Eq)]
pub enum KeyboardAction {
    Next,
    Prev,
    PausePlay,
    Faster,
    Slower,
    ToEnd,
    Exit,
    None,
}

impl Interaction {
    pub const fn default() -> Self {
        Self {
            len: 0,
            idx: 0,
            forward: true,
            spf: Duration::SECOND,
            playing: false,
        }
    }

    pub fn new(len: usize) -> Self {
        Self {
            len,
            idx: 0,
            forward: true,
            spf: Duration::SECOND,
            playing: true,
        }
    }
    pub fn reset(&mut self, len: usize) {
        *self = Self::new(len);
    }
    pub fn done(&mut self) {
        self.len = self.idx;
    }
    pub fn is_done(&mut self) -> bool {
        self.idx >= self.len
    }

    pub fn prev(&mut self) -> bool {
        self.forward = false;
        let r = self.idx > 0;
        self.idx = self.idx.saturating_sub(1);
        r
    }
    pub fn next(&mut self) -> bool {
        self.forward = true;
        if self.idx + 1 < self.len {
            self.idx += 1;
            true
        } else {
            false
        }
    }
    pub fn step(&mut self) -> bool {
        if self.forward {
            self.next()
        } else {
            self.prev()
        }
    }
    pub fn toend(&mut self) {
        self.idx = self.len - 1;
    }
    pub fn get(&self) -> usize {
        self.idx
    }
    pub fn faster(&mut self) {
        self.spf = self.spf.div_f32(1.5);
    }
    pub fn slower(&mut self) {
        self.spf = self.spf.mul_f32(1.5);
    }
    pub fn pauseplay(&mut self) {
        self.playing = !self.playing;
    }

    #[cfg(feature = "bin")]
    pub fn wait(&mut self) {
        // use crate::canvas::sdl::wait_for_key;

        // match wait_for_key(if self.playing {
        //     self.spf
        // } else {
        //     Duration::MAX
        // }) {
        //     KeyboardAction::Next => {
        //         self.next();
        //     }
        //     KeyboardAction::Prev => {
        //         self.prev();
        //     }
        //     KeyboardAction::PausePlay => self.pauseplay(),
        //     KeyboardAction::Faster => self.faster(),
        //     KeyboardAction::Slower => self.slower(),
        //     KeyboardAction::ToEnd => self.toend(),
        //     KeyboardAction::Exit => {
        //         eprintln!("Aborted by user!");
        //         std::process::exit(1);
        //     }
        //     KeyboardAction::None => {
        //         if self.playing {
        //             self.step();
        //         }
        //     }
        // }
    }
    #[cfg(feature = "wasm")]
    pub fn wait(&mut self) {}
}
