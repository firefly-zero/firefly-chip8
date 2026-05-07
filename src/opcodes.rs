#[expect(clippy::upper_case_acronyms)]
pub enum Opcode {
    /// 00E0. `cls`: Clear Screen
    I00E0,
    /// 00EE. `ret`: Return
    I00EE,
    /// 1nnn. `jp addr`: Jump to `addr`
    I1NNN(u16),
    /// 2nnn. `call addr`: Call routine at `addr`
    I2NNN(u16),
    /// 3xnn. `se Vn kk`: Skip if `Vn` equals `kk`
    I3XNN(u8, u8),
    /// 4xnn. `sne Vn kk`: Skip if `Vn` does not equal `kk`
    I4XNN(u8, u8),
    /// 5xy0. `se Vx Vy`: Skip if `Vn` equals `Vy`
    I5XY0(u8, u8),
    /// 6xnn. `ld Vx, kk`: Loads a literal value `kk` into `Vx`
    I6XNN(u8, u8),
    /// 7xnn. `add Vn, kk`: Add `kk` to register `Vx`
    I7XNN(u8, u8),
    /// 8xy0. `ld Vx, Vy`: Loads register `Vy` into `Vx`
    I8XY0(u8, u8),
    /// 8xy1. `or Vx, Vy`: Bitwise OR the value in `Vy` with register `Vx`
    I8XY1(u8, u8),
    /// 8xy2. `and Vx, Vy`: Bitwise AND the value in `Vy` with register `Vx`
    I8XY2(u8, u8),
    /// 8xy3. `xor Vx, Vy`: Bitwise XOR the value in `Vy` with register `Vx`
    I8XY3(u8, u8),
    /// 8xy4. `add Vx, Vy`: Add the value in `Vy` to register `Vx`
    I8XY4(u8, u8),
    /// 8xy5. `xor Vx, Vy`: Subtract the value in `Vy` from register `Vx`
    I8XY5(u8, u8),
    /// 8xy6. `shr Vx [, Vy]`: Shift `Vx` to the right with the value in `Vy`
    I8XY6(u8, u8),
    /// 8xy7. `subn Vx, Vy`: Subtract the value in `Vy` from `Vx`, no carry
    I8XY7(u8, u8),
    /// 8xyE. `shl Vx [, Vy]`: Shift `Vx` to the left with the value in `Vy`
    I8XYE(u8, u8),
    /// 9xy0. `sne Vx Vy`: Skip if `Vn` does not equal `Vy`
    I9XY0(u8, u8),
    /// Annn. `ld I, nnn`: Loads `nnn` into register `I`
    IANNN(u16),
    /// Bnnn. `jp v0, addr`: Jump to `v0 + addr`
    IBNNN(u16),
    /// Cxnn. `rnd Vn, kk`: random number AND `kk` into `Vn`
    ICXNN(u8, u8),
    /// Dxyn. `drw Vx, Vy, n`: Draw a sprite of `n` rows at `Vx,Vy`
    IDXYN(u8, u8, u8),
    /// Ex9e. `skp Vn`: Skip if key in `Vn` pressed
    IEX9E(u8),
    /// Exa1. `sknp Vn`: Skip if key in `Vn` not pressed
    IEXA1(u8),
    /// Fx07. `ld Vx, DT`: Loads the delay timer into register `Vx`
    IFX07(u8),
    /// Fx0a. `ld Vn, K`: loads a key pressed into `Vn`
    IFX0A(u8),
    /// Fx15. `delay Vx`: Loads register `Vx` into the delay timer
    IFX15(u8),
    /// Fx18. `sound Vx`: Loads register `Vx` into the sound timer
    IFX18(u8),
    /// Fx1E. `add I, Vn`: Add the value in `Vn` to register `I`
    IFX1E(u8),
    /// Fx29. `hex Vx`: Loads the 8&times;5 font sprite of `Vx` into `I`
    IFX29(u8),
    /// Fx33. `bcd Vx`: Load BCD value of `Vx` into `I` to `I+2`
    IFX33(u8),
    /// Fx55. `stor Vx`: Stores `V0` through `Vx` to the address in `I`
    IFX55(u8),
    /// Fx65. `rstr Vx`: Restores `V0` through `Vx` from the address in `I`
    IFX65(u8),
}

impl Opcode {
    pub fn new(opcode: u16) -> Option<Opcode> {
        let x = ((opcode >> 8) & 0x000F) as u8;
        let y = ((opcode >> 4) & 0x000F) as u8;
        let n = (opcode & 0x000F) as u8;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => Some(Opcode::I00E0),
                0x00EE => Some(Opcode::I00EE),
                _ => None,
            },
            0x1000 => Some(Opcode::I1NNN(nnn)),
            0x2000 => Some(Opcode::I2NNN(nnn)),
            0x3000 => Some(Opcode::I3XNN(x, nn)),
            0x4000 => Some(Opcode::I4XNN(x, nn)),
            0x5000 => match n {
                0x0 => Some(Opcode::I5XY0(x, y)),
                _ => None,
            },
            0x6000 => Some(Opcode::I6XNN(x, nn)),
            0x7000 => Some(Opcode::I7XNN(x, nn)),
            0x8000 => match n {
                0x0 => Some(Opcode::I8XY0(x, y)),
                0x1 => Some(Opcode::I8XY1(x, y)),
                0x2 => Some(Opcode::I8XY2(x, y)),
                0x3 => Some(Opcode::I8XY3(x, y)),
                0x4 => Some(Opcode::I8XY4(x, y)),
                0x5 => Some(Opcode::I8XY5(x, y)),
                0x6 => Some(Opcode::I8XY6(x, y)),
                0x7 => Some(Opcode::I8XY7(x, y)),
                0xE => Some(Opcode::I8XYE(x, y)),
                _ => None,
            },
            0x9000 => match n {
                0x0 => Some(Opcode::I9XY0(x, y)),
                _ => None,
            },
            0xA000 => Some(Opcode::IANNN(nnn)),
            0xB000 => Some(Opcode::IBNNN(nnn)),
            0xC000 => Some(Opcode::ICXNN(x, nn)),
            0xD000 => Some(Opcode::IDXYN(x, y, n)),
            0xE000 => match nn {
                0x9E => Some(Opcode::IEX9E(x)),
                0xA1 => Some(Opcode::IEXA1(x)),
                _ => None,
            },
            0xF000 => match nn {
                0x07 => Some(Opcode::IFX07(x)),
                0x0A => Some(Opcode::IFX0A(x)),
                0x15 => Some(Opcode::IFX15(x)),
                0x18 => Some(Opcode::IFX18(x)),
                0x1E => Some(Opcode::IFX1E(x)),
                0x29 => Some(Opcode::IFX29(x)),
                0x33 => Some(Opcode::IFX33(x)),
                0x55 => Some(Opcode::IFX55(x)),
                0x65 => Some(Opcode::IFX65(x)),
                _ => None,
            },
            _ => None,
        }
    }
}
