#![no_std]
#![no_main]
mod config;

extern crate alloc;

use config::*;
use core::mem::MaybeUninit;
use firefly_rust::*;
use rsc8::chip8::Chip8;

static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();
const SCALE: i32 = 3;
const SCREEN_WIDTH: i32 = rsc8::chip8::SCREEN_WIDTH as i32;
const SCREEN_HEIGHT: i32 = rsc8::chip8::SCREEN_HEIGHT as i32;
const AREA_WIDTH: i32 = SCREEN_WIDTH * SCALE;
const AREA_HEIGHT: i32 = SCREEN_HEIGHT * SCALE;

struct Rng;

impl Iterator for Rng {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        Some(get_random() as u16)
    }
}

struct State {
    chip8: Chip8<Rng>,
    screen: [u8; 64 * 32],
    config: Config,
    plays: bool,
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

    let state = State {
        chip8,
        screen: [0; 32 * 64],
        config: Config::load().unwrap_or_default(),
        plays: false,
    };
    clear_screen(Color::Black);
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

    // Play audio.
    if state.plays && chip8.sound_timer == 0 {
        audio::OUT.clear();
        state.plays = false;
    } else if !state.plays && chip8.sound_timer > 0 {
        // Other emulators play a sine wave but the current firefly-audio
        // implementation sharply cuts the audio as soon as it's stopped
        // resulting in a sharp audio spike. This spike is less apparent
        // with a triangle wave.
        audio::OUT.add_triangle(audio::Freq::C4, 0.);
        state.plays = true;
    }

    // Advance the virtual CHIP-8 CPU.
    for _ in 0..state.config.speed {
        chip8.tick().unwrap();
    }
    chip8.tick_timer();
}

fn handle_input(state: &mut State) {
    let chip8 = &mut state.chip8;

    let pad = read_pad(Peer::COMBINED).unwrap_or_default();
    let dpad = pad.as_dpad8();
    let btns = read_buttons(Peer::COMBINED);
    for (i, input) in state.config.inputs.iter().enumerate() {
        let Some((_peer, input)) = input else {
            continue;
        };
        chip8.keypad[i] = match input {
            Input::L => dpad.left,
            Input::R => dpad.right,
            Input::U => dpad.up,
            Input::D => dpad.down,
            Input::S => btns.s,
            Input::E => btns.e,
            Input::W => btns.w,
            Input::N => btns.n,
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    let state = get_state();
    let chip8 = &mut state.chip8;

    draw_rect(
        Point::new((WIDTH - AREA_WIDTH) / 2, (HEIGHT - AREA_HEIGHT) / 2),
        Size::new(AREA_WIDTH, AREA_HEIGHT),
        Style::solid(Color::White),
    );

    for (i, set) in chip8.screen.iter().enumerate() {
        if *set {
            state.screen[i] = 3;
        } else {
            state.screen[i] = state.screen[i].saturating_sub(1);
        }
    }

    let size = Size::new(SCALE, SCALE);
    for (color, i) in state.screen.iter().zip(0..) {
        let color = match color {
            0 => continue,
            1 => Color::LightBlue,
            2 => Color::Blue,
            _ => Color::DarkBlue,
        };
        let x = (WIDTH - AREA_WIDTH) / 2 + (i % SCREEN_WIDTH) * SCALE;
        let y = (HEIGHT - AREA_HEIGHT) / 2 + (i / SCREEN_WIDTH) * SCALE;
        let p = Point::new(x, y);
        draw_rect(p, size, Style::solid(color));
    }
}
