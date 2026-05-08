use alloc::boxed::Box;
use alloc::vec;
use firefly_rust::get_random;

use crate::opcodes::Opcode;

pub const MEMORY_SIZE: usize = 4096;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

/// The built-in font.
///
/// The sprite data represents the hexadecimal numbers from 0 through F.
///
/// https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#font
const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    mem: Box<[u8]>,
    program_cnt: u16,
    reg_v: [u8; 16],
    reg_i: u16,
    delay_timer: u8,
    pub sound_timer: u8,
    stack: [u16; 16],
    stack_ptr: u8,
    pub input: [bool; 16],
    pub screen: Box<[bool]>,
}

impl Chip8 {
    pub fn new(rom: &[u8]) -> Result<Self, &'static str> {
        pub const ROM_START: usize = 512;

        let mut memory = vec![0; MEMORY_SIZE];
        memory[..FONT.len()].copy_from_slice(&FONT);
        let max_size = MEMORY_SIZE - ROM_START;
        if rom.len() > max_size {
            return Err("ROM is too large");
        }
        let rom_end = ROM_START + rom.len();
        memory[ROM_START..rom_end].copy_from_slice(rom);

        let chip8 = Self {
            mem: memory.into_boxed_slice(),
            program_cnt: 0x200,
            reg_v: [0; _],
            reg_i: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; _],
            stack_ptr: 0,
            input: [false; _],
            screen: vec![false; SCREEN_WIDTH * SCREEN_HEIGHT].into_boxed_slice(),
        };
        Ok(chip8)
    }

    /// Run the update cycle.
    pub fn update(&mut self, steps: u16) -> Result<(), &'static str> {
        for _ in 0..steps {
            self.step()?;
        }
        self.delay_timer = self.delay_timer.saturating_sub(1);
        self.sound_timer = self.sound_timer.saturating_sub(1);
        Ok(())
    }

    // Execute the next opcode.
    fn step(&mut self) -> Result<(), &'static str> {
        let opcode = self.next()?;
        let Some(opcode) = Opcode::new(opcode) else {
            return Err("invalid opcode");
        };
        self.exec(opcode)
    }

    /// Get the next opcode to execute.
    fn next(&mut self) -> Result<u16, &'static str> {
        let pc = self.program_cnt as usize;
        if pc + 1 >= MEMORY_SIZE {
            return Err("program counter out of bounds");
        }
        let high_byte = self.mem[pc];
        let low_byte = self.mem[pc + 1];

        self.program_cnt = self
            .program_cnt
            .checked_add(2)
            .ok_or("program counter overflow")?;

        Ok(((high_byte as u16) << 8) | low_byte as u16)
    }

    fn load(&self, addr: usize) -> Result<u8, &'static str> {
        self.mem.get(addr).copied().ok_or("memory out of bounds")
    }

    fn write_memory(&mut self, address: usize, value: u8) -> Result<(), &'static str> {
        if let Some(cell) = self.mem.get_mut(address) {
            *cell = value;
            Ok(())
        } else {
            Err("memory out of bounds")
        }
    }

    /// Skip the next opcode.
    fn skip(&mut self) -> Result<(), &'static str> {
        self.program_cnt = self
            .program_cnt
            .checked_add(2)
            .ok_or("program counter overflow")?;
        Ok(())
    }

    /// Step back to re-execute the current opcode.
    fn repeat(&mut self) -> Result<(), &'static str> {
        self.program_cnt = self
            .program_cnt
            .checked_sub(2)
            .ok_or("program counter overflow")?;
        Ok(())
    }

    /// Push the value on the stack.
    fn push(&mut self, value: u16) -> Result<(), &'static str> {
        let stack_index = self.stack_ptr as usize;
        if stack_index >= self.stack.len() {
            return Err("stack overflow");
        }
        self.stack[stack_index] = value;
        self.stack_ptr += 1;
        Ok(())
    }

    /// Pop a value from the stack.
    fn pop(&mut self) -> Result<u16, &'static str> {
        if self.stack_ptr == 0 {
            return Err("trying to pop a value from an empty stack");
        }
        self.stack_ptr -= 1;
        Ok(self.stack[self.stack_ptr as usize])
    }

    fn get_key(&self, register_index: u8) -> Result<bool, &'static str> {
        let key_index = self.reg_v[register_index as usize];
        if key_index as usize >= self.input.len() {
            return Err("invalid key index");
        }
        Ok(self.input[key_index as usize])
    }

    fn exec(&mut self, opcode: Opcode) -> Result<(), &'static str> {
        match opcode {
            Opcode::I00E0 => {
                self.screen.fill(false);
            }
            Opcode::I00EE => {
                self.program_cnt = self.pop()?;
            }
            Opcode::I1NNN(nnn) => {
                self.program_cnt = nnn;
            }
            Opcode::I2NNN(nnn) => {
                self.push(self.program_cnt)?;
                self.program_cnt = nnn;
            }
            Opcode::I3XNN(x, nn) => {
                if self.reg_v[x as usize] == nn {
                    self.skip()?;
                }
            }
            Opcode::I4XNN(x, nn) => {
                if self.reg_v[x as usize] != nn {
                    self.skip()?;
                }
            }
            Opcode::I5XY0(x, y) => {
                if self.reg_v[x as usize] == self.reg_v[y as usize] {
                    self.skip()?;
                }
            }
            Opcode::I6XNN(x, nn) => {
                self.reg_v[x as usize] = nn;
            }
            Opcode::I7XNN(x, nn) => {
                self.reg_v[x as usize] = self.reg_v[x as usize].wrapping_add(nn);
            }
            Opcode::I8XY0(x, y) => {
                self.reg_v[x as usize] = self.reg_v[y as usize];
            }
            Opcode::I8XY1(x, y) => {
                self.reg_v[x as usize] |= self.reg_v[y as usize];
                self.reg_v[0xF] = 0;
            }
            Opcode::I8XY2(x, y) => {
                self.reg_v[x as usize] &= self.reg_v[y as usize];
                self.reg_v[0xF] = 0;
            }
            Opcode::I8XY3(x, y) => {
                self.reg_v[x as usize] ^= self.reg_v[y as usize];
                self.reg_v[0xF] = 0;
            }
            Opcode::I8XY4(x, y) => {
                let (result, carry) =
                    self.reg_v[x as usize].overflowing_add(self.reg_v[y as usize]);
                self.reg_v[x as usize] = result;
                self.reg_v[0xF] = carry as u8;
            }
            Opcode::I8XY5(x, y) => {
                let (result, carry) =
                    self.reg_v[x as usize].overflowing_sub(self.reg_v[y as usize]);
                self.reg_v[x as usize] = result;
                self.reg_v[0xF] = !carry as u8;
            }
            Opcode::I8XY6(x, y) => {
                self.reg_v[x as usize] = self.reg_v[y as usize];
                let lsb = self.reg_v[x as usize] & 1;
                self.reg_v[x as usize] >>= 1;
                self.reg_v[0xF] = lsb;
            }
            Opcode::I8XY7(x, y) => {
                let (result, carry) =
                    self.reg_v[y as usize].overflowing_sub(self.reg_v[x as usize]);
                self.reg_v[x as usize] = result;
                self.reg_v[0xF] = !carry as u8;
            }
            Opcode::I8XYE(x, y) => {
                self.reg_v[x as usize] = self.reg_v[y as usize];
                let msb = self.reg_v[x as usize] >> 7;
                self.reg_v[x as usize] <<= 1;
                self.reg_v[0xF] = msb;
            }
            Opcode::I9XY0(x, y) => {
                if self.reg_v[x as usize] != self.reg_v[y as usize] {
                    self.skip()?;
                }
            }
            Opcode::IANNN(nnn) => {
                self.reg_i = nnn;
            }
            Opcode::IBNNN(nnn) => {
                self.program_cnt = nnn + self.reg_v[0] as u16;
            }
            Opcode::ICXNN(x, nn) => {
                let random = get_random() as u16;
                self.reg_v[x as usize] = random as u8 & nn;
            }
            Opcode::IDXYN(x, y, n) => {
                let vx = self.reg_v[x as usize] % SCREEN_WIDTH as u8;
                let vy = self.reg_v[y as usize] % SCREEN_HEIGHT as u8;
                self.reg_v[0xF] = 0;
                for row in 0..n {
                    let screen_y = vy + row;
                    if screen_y >= SCREEN_HEIGHT as u8 {
                        break;
                    }
                    let sprite_address = self.reg_i as usize + row as usize;
                    let sprite_row = self.load(sprite_address)?;
                    for col in 0..8 {
                        let screen_x = vx + col;
                        if screen_x >= SCREEN_WIDTH as u8 {
                            break;
                        }
                        let sprite_pixel = (sprite_row & (0b1000_0000 >> col)) != 0;
                        let screen_pixel_index =
                            screen_x as usize + screen_y as usize * SCREEN_WIDTH;
                        let screen_pixel = self.screen[screen_pixel_index];
                        if sprite_pixel && screen_pixel {
                            self.reg_v[0xF] = 1;
                        }
                        self.screen[screen_pixel_index] ^= sprite_pixel;
                    }
                }
            }
            Opcode::IEX9E(x) => {
                if self.get_key(x)? {
                    self.skip()?;
                }
            }
            Opcode::IEXA1(x) => {
                if !self.get_key(x)? {
                    self.skip()?;
                }
            }
            Opcode::IFX07(x) => {
                self.reg_v[x as usize] = self.delay_timer;
            }
            Opcode::IFX0A(x) => {
                let mut any_pressed = false;
                for (key_code, &key_pressed) in self.input.iter().enumerate() {
                    if key_pressed {
                        any_pressed = true;
                        self.reg_v[x as usize] = key_code as u8;
                        break;
                    }
                }
                if !any_pressed {
                    self.repeat()?;
                }
            }
            Opcode::IFX15(x) => {
                self.delay_timer = self.reg_v[x as usize];
            }
            Opcode::IFX18(x) => {
                self.sound_timer = self.reg_v[x as usize];
            }
            Opcode::IFX1E(x) => {
                self.reg_i = self.reg_i.wrapping_add(self.reg_v[x as usize] as u16);
            }
            Opcode::IFX29(x) => {
                self.reg_i = (self.reg_v[x as usize] * 5) as u16;
            }
            Opcode::IFX33(x) => {
                let hundreds = self.reg_v[x as usize] / 100;
                let tens = (self.reg_v[x as usize] / 10) % 10;
                let ones = self.reg_v[x as usize] % 10;
                self.write_memory(self.reg_i as usize, hundreds)?;
                self.write_memory(self.reg_i as usize + 1, tens)?;
                self.write_memory(self.reg_i as usize + 2, ones)?;
            }
            Opcode::IFX55(x) => {
                for index in 0..=x {
                    let address = self.reg_i as usize + index as usize;
                    self.write_memory(address, self.reg_v[index as usize])?;
                }
                self.reg_i = self.reg_i.wrapping_add(x as u16 + 1);
            }
            Opcode::IFX65(x) => {
                for index in 0..=x {
                    let address = self.reg_i as usize + index as usize;
                    self.reg_v[index as usize] = self.load(address)?;
                }
                self.reg_i = self.reg_i.wrapping_add(x as u16 + 1);
            }
        }
        Ok(())
    }
}
