# Chip-8 Emulator

This is a simple implementation of a [Chip-8](https://en.wikipedia.org/wiki/CHIP-8) emulator written in Rust, based on [Austin Morlan's](austinmorlan.com) Chip-8 emulator.

## Build
The emulator is very easy to build as long as you have [cargo](https://www.rust-lang.org/tools/install) and [SDL2](https://www.libsdl.org/) installed.
To build the emulator run: 
```shell
cargo build --release
```
And to run the emulator:
```shell
target/release/chip8 10 1 games/test_opcode.ch8
```
The arguments are as follows:
```
./chip8 <Window Scale> <Cpu Cycle Delay(ms)> <Rom path>
```
For more chip8 roms check out [dmatlack's repo](https://github.com/dmatlack/chip8/tree/master/roms/games)

### Issues
This emulator is not perfect and still has some major flaws. Most of these stem from integer overflows and indices being out of bounds causing crashes.  