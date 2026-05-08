# firefly-chip8

[CHIP-8](https://en.wikipedia.org/wiki/CHIP-8) emulator for [Firefly Zero](https://fireflyzero.com/).

## Usage

First, create `config.txt`. For example:

```text
speed=14
2=1U
C=2U
D=SN
```

This config tells the emulator:

* `speed=14`: run 14 instructions per update
* `2=1U`: read CHIP-8 `2` button from Firefly `up` of the first peer.
* `C=2U`: read CHIP-8 `C` button from Firefly `up` of the second peer.
* `D=SN`: read CHIP-8 `D` button from Firefly `N` button of the shared peer (pressed if any peer presses it).

Include the emulator, the ROM, and the config in `firefly.toml`:

```toml
# ...

[files]
_bin = { path = "main.wasm", url = "https://github.com/firefly-zero/firefly-chip8/releases/latest/download/main.wasm", copy = true }
main = { path = "pong.ch8", copy = true }
config = { path = "config.txt", copy = true }
```

Now you can `ff build` and play the game.

If you need to update the interpreter to the latest version, remove `main.wasm` from the project root and `ff build` again.
