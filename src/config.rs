use firefly_rust::*;

#[derive(Clone, Copy)]
pub enum Input {
    L,
    R,
    U,
    D,
    S,
    E,
    W,
    N,
}

pub struct Config {
    /// How many CHIP-8 instructions to run per update.
    pub speed: u16,
    pub inputs: [Option<(Peer, Input)>; 16],
}

impl Config {
    pub fn load() -> Option<Self> {
        let Some(file) = load_file_buf("config") else {
            log_debug("config not found");
            return None;
        };
        let mut config = Config::default();
        let peers = get_peers().as_vec();
        for line in file.as_bytes().split(|c| *c == b'\n') {
            if line.is_empty() {
                continue;
            }
            if let Some(suffix) = line.strip_prefix(b"speed=") {
                let suffix = unsafe { str::from_utf8_unchecked(suffix) };
                let Ok(speed) = suffix.parse::<u16>() else {
                    log_error("failed to parse speed");
                    return None;
                };
                config.speed = speed;
            }
            let Some((idx, peer, input)) = parse_input(line, &peers) else {
                continue;
            };
            config.inputs[idx] = Some((peer, input));
        }
        Some(config)
    }
}

fn parse_input(line: &[u8], peers: &[Peer]) -> Option<(usize, Peer, Input)> {
    if line.len() != 4 {
        log_error("invalid line length");
        return None;
    }
    let idx: usize = match line[0] {
        b'0' => 0,
        b'1' => 1,
        b'2' => 2,
        b'3' => 3,
        b'4' => 4,
        b'5' => 5,
        b'6' => 6,
        b'7' => 7,
        b'8' => 8,
        b'9' => 9,
        b'A' | b'a' => 0xA,
        b'B' | b'b' => 0xB,
        b'C' | b'c' => 0xC,
        b'D' | b'd' => 0xD,
        b'E' | b'e' => 0xE,
        b'F' | b'f' => 0xF,
        _ => {
            log_error("invalid CHIP-8 button name");
            return None;
        }
    };
    if line[1] != b'=' {
        log_error("cannot find '='");
        return None;
    }
    let peer = match line[2] {
        b'1' => peers.first()?,
        b'2' => peers.get(1)?,
        b'3' => peers.get(2)?,
        b'4' => peers.get(3)?,
        b'S' => &Peer::COMBINED,
        _ => {
            log_error("invalid peer");
            return None;
        }
    };
    let input = match line[3] {
        b'L' => Input::L,
        b'R' => Input::R,
        b'U' => Input::U,
        b'D' => Input::D,
        b'S' => Input::S,
        b'E' => Input::E,
        b'W' => Input::W,
        b'N' => Input::N,
        _ => {
            log_error("invalid Firefly button name");
            return None;
        }
    };
    Some((idx, *peer, input))
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // 17 ticks with 60 FPS result in ~1MHz.
            speed: 17,
            inputs: [
                None,                             // 0
                None,                             // 1
                Some((Peer::COMBINED, Input::U)), // 2
                None,                             // 3
                Some((Peer::COMBINED, Input::L)), // 4
                None,                             // 5
                Some((Peer::COMBINED, Input::R)), // 6
                None,                             // 7
                Some((Peer::COMBINED, Input::D)), // 8
                None,                             // 9
                Some((Peer::COMBINED, Input::S)), // A
                Some((Peer::COMBINED, Input::E)), // B
                None,                             // C
                None,                             // D
                None,                             // E
                None,                             // F
            ],
        }
    }
}
