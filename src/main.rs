#![no_std]
#![no_main]
mod config;
mod interpreter;
mod opcodes;

extern crate alloc;

use config::*;
use core::mem::MaybeUninit;
use firefly_rust::*;
use interpreter::*;

static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();
const SCALE: i32 = 3;
const SCREEN_WIDTH: i32 = interpreter::SCREEN_WIDTH as i32;
const SCREEN_HEIGHT: i32 = interpreter::SCREEN_HEIGHT as i32;
const AREA_WIDTH: i32 = SCREEN_WIDTH * SCALE;
const AREA_HEIGHT: i32 = SCREEN_HEIGHT * SCALE;

struct State {
    chip8: Chip8,
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
    let Some(file) = load_file_buf("main") else {
        log_error("main file not found");
        panic!();
    };
    let Ok(chip8) = Chip8::new(file.as_bytes()) else {
        log_error("invalid rom");
        panic!();
    };

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
    let res = chip8.update(state.config.speed);
    if let Err(err) = res {
        log_error(err);
        panic!();
    }
}

fn handle_input(state: &mut State) {
    let chip8 = &mut state.chip8;
    for (i, input) in state.config.inputs.iter().enumerate() {
        let Some((peer, input)) = input else {
            continue;
        };
        chip8.input[i] = match input {
            Input::L => read_dpad(*peer).left,
            Input::R => read_dpad(*peer).right,
            Input::U => read_dpad(*peer).up,
            Input::D => read_dpad(*peer).down,
            Input::S => read_buttons(*peer).s,
            Input::E => read_buttons(*peer).e,
            Input::W => read_buttons(*peer).w,
            Input::N => read_buttons(*peer).n,
        }
    }
}

fn read_dpad(peer: Peer) -> DPad8 {
    read_pad(peer).unwrap_or_default().as_dpad8()
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
