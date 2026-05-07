#![no_std]
#![no_main]
extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::mem::MaybeUninit;
use firefly_rust::*;
use rsc8::chip8::{Chip8, SCREEN_HEIGHT, SCREEN_WIDTH};

static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();

struct Rng;

impl Iterator for Rng {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        Some(get_random() as u16)
    }
}

struct State {
    chip8: Chip8<Rng>,
}

fn get_state() -> &'static mut State {
    #[allow(static_mut_refs)]
    unsafe {
        STATE.assume_init_mut()
    }
}

#[unsafe(no_mangle)]
extern "C" fn boot() {
    let mut chip8 = Chip8::new(Rng);
    chip8.load_fontset();
    let Some(file) = load_file_buf("main") else {
        log_error("main file not found");
        panic!();
    };
    let res = chip8.load_rom(file.as_bytes());
    if res.is_err() {
        log_error("invalid rom");
        panic!();
    }
    let state = State { chip8 };
    #[allow(static_mut_refs)]
    unsafe {
        STATE.write(state)
    };
}

#[unsafe(no_mangle)]
extern "C" fn update() {
    let state = get_state();
    handle_input(state);

    let chip8 = &mut state.chip8;
    for _ in 0..8 {
        chip8.tick().unwrap();
    }
    chip8.tick_timer();
}

fn handle_input(state: &mut State) {
    let pad = read_pad(Peer::COMBINED).unwrap_or_default();
    let dpad = pad.as_dpad8();
    // ...

    let btns = read_buttons(Peer::COMBINED);
    // ...
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    let state = get_state();
    let chip8 = &mut state.chip8;
    if !chip8.draw_flag {
        return;
    }
    chip8.draw_flag = false;
    clear_screen(Color::White);
    for (set, i) in chip8.screen.iter().zip(0..) {
        if !set {
            continue;
        }
        let x = (WIDTH - SCREEN_WIDTH as i32) / 2 + i % SCREEN_WIDTH as i32;
        let y = (HEIGHT - SCREEN_HEIGHT as i32) / 2 + i / SCREEN_WIDTH as i32;
        let p = Point::new(x, y);
        draw_point(p, Color::DarkBlue);
    }
    // ...
}
