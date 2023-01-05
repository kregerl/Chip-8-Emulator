use std::{fs::File, io::Read};
use std::cmp::min;

use rand::Rng;
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;

use crate::{VIDEO_HEIGHT, VIDEO_WIDTH};

const START_ADDRESS: usize = 0x200;
const FONTSET_START_ADDRESS: usize = 0x50;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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

pub struct Cpu {
    registers: [u8; 16],
    memory: [u8; 4096],
    index: u16,
    program_counter: u16,
    stack: [u16; 16],
    stack_pointer: u16,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    pub video: [u32; VIDEO_WIDTH * VIDEO_HEIGHT],
}

#[derive(Debug)]
enum Operation {
    Cls00E0(u16),
    Ret00EE(u16),
    Jp1nnn(u16),
    Call2nnn(u16),
    Se3xkk(u16),
    Sne4xkk(u16),
    Se5xy0(u16),
    Ld6xkk(u16),
    Add7xkk(u16),
    Ld8xy0(u16),
    Or8xy1(u16),
    And8xy2(u16),
    Xor8xy3(u16),
    Add8xy4(u16),
    Sub8xy5(u16),
    Shr8xy6(u16),
    Subn8xy7(u16),
    Shl8xyE(u16),
    Sne9xy0(u16),
    LdAnnn(u16),
    JpBnnn(u16),
    RndCxkk(u16),
    DrwDxyn(u16),
    SkpEx9e(u16),
    SknpExA1(u16),
    LdFx07(u16),
    LdFx0a(u16),
    LdFx15(u16),
    LdFx18(u16),
    AddFx1e(u16),
    LdFx29(u16),
    LdFx33(u16),
    LdFx55(u16),
    LdFx65(u16),
    Null(u16),
}

impl Cpu {
    pub fn new(rom_path: &str) -> Self {
        let mut file = File::open(rom_path).expect(&format!("Error opening rom file {}", rom_path));
        let num_bytes = file.metadata().expect("Unable to get rom metadata").len();
        let mut buffer = vec![0; num_bytes as usize];
        file.read(&mut buffer).expect("Error reading rom file");
        let mut memory = [0; 4096];
        // Load rom into memory from 0x200 onward
        for i in 0..buffer.len() {
            memory[START_ADDRESS + i] = buffer[i];
        }

        // Load fontset at 0x50
        for i in 0..FONTSET_SIZE {
            memory[FONTSET_START_ADDRESS + i] = FONTSET[i];
        }

        Self {
            registers: [0; 16],
            memory,
            index: 0,
            program_counter: START_ADDRESS as u16,
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            video: [0; 64 * 32],
        }
    }

    fn execute(&mut self, op: Operation) {
        match op {
            Operation::Cls00E0(_) => self.video.fill_with(|| 0x0),
            Operation::Ret00EE(_) => {
                self.stack_pointer -= 1;
                self.program_counter = self.stack[self.stack_pointer as usize];
            }
            Operation::Jp1nnn(opcode) => {
                let addr = opcode & 0x0FFF;
                self.program_counter = addr;
            }
            Operation::Call2nnn(opcode) => {
                let addr = opcode & 0x0FFF;
                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.stack_pointer += 1;
                self.program_counter = addr;
            }
            Operation::Se3xkk(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let byte = (opcode & 0x00FF) as u8;
                if self.registers[vx as usize] == byte {
                    self.program_counter += 2;
                }
            }
            Operation::Sne4xkk(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let byte = (opcode & 0x00FF) as u8;
                if self.registers[vx as usize] != byte {
                    self.program_counter += 2;
                }
            }
            Operation::Se5xy0(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let vy = ((opcode & 0x00F0) >> 4) as u8;
                if self.registers[vx as usize] == self.registers[vy as usize] {
                    self.program_counter += 2;
                }
            }
            Operation::Ld6xkk(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let byte = (opcode & 0x00FF) as u8;
                self.registers[vx as usize] = byte;
            }
            Operation::Add7xkk(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let byte = (opcode & 0x00FF) as u8;
                let sum = self.registers[vx as usize] as u16 + byte as u16;
                self.registers[vx as usize] = (sum & 0xFF) as u8;
            }
            Operation::Ld8xy0(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let vy = ((opcode & 0x00F0) >> 4) as u8;
                self.registers[vx as usize] = self.registers[vy as usize];
            }
            Operation::Or8xy1(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let vy = ((opcode & 0x00F0) >> 4) as u8;
                self.registers[vx as usize] |= self.registers[vy as usize];
            }
            Operation::And8xy2(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let vy = ((opcode & 0x00F0) >> 4) as u8;
                self.registers[vx as usize] &= self.registers[vy as usize];
            }
            Operation::Xor8xy3(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let vy = ((opcode & 0x00F0) >> 4) as u8;
                self.registers[vx as usize] ^= self.registers[vy as usize];
            }
            Operation::Add8xy4(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let vy = ((opcode & 0x00F0) >> 4) as u8;

                let sum = self.registers[vx as usize] as u16 + self.registers[vy as usize] as u16;
                if sum > 255 {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                self.registers[vx as usize] = (sum & 0xFF) as u8;
            }
            Operation::Sub8xy5(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let vy = ((opcode & 0x00F0) >> 4) as u8;

                if self.registers[vx as usize] > self.registers[vy as usize] {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                self.registers[vx as usize] = Self::safe_subtract(self.registers[vx as usize], self.registers[vy as usize]);
            }
            Operation::Shr8xy6(opcode) => {
                let vx = (opcode & 0x0F00) >> 8 as u8;
                self.registers[0xF] = self.registers[vx as usize] * 0x1;
                self.registers[vx as usize] >>= 1;
            }
            Operation::Subn8xy7(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let vy = ((opcode & 0x00F0) >> 4) as u8;

                if self.registers[vy as usize] > self.registers[vx as usize] {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                self.registers[vx as usize] =
                    self.registers[vy as usize] - self.registers[vx as usize];
            }
            Operation::Shl8xyE(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                self.registers[0xF] = (self.registers[vx as usize] & 0x80) >> 7;
                self.registers[vx as usize] <<= 1;
            }
            Operation::Sne9xy0(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let vy = ((opcode & 0x00F0) >> 4) as u8;
                if self.registers[vx as usize] != self.registers[vy as usize] {
                    self.program_counter += 2;
                }
            }
            Operation::LdAnnn(opcode) => {
                let addr = opcode & 0x0FFF;
                self.index = addr;
            }
            Operation::JpBnnn(opcode) => {
                let addr = opcode & 0x0FFF;
                self.program_counter = self.registers[0] as u16 + addr;
            }
            Operation::RndCxkk(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let byte = (opcode & 0x00FF) as u8;

                self.registers[vx as usize] = Self::get_random_number() & byte;
            }
            Operation::DrwDxyn(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let vy = ((opcode & 0x00F0) >> 4) as u8;
                let height = opcode & 0x000F;
                // println!("Opcode: {} -- vx: {}, vy: {}, height: {}", opcode, vx, vy, height);

                let x_pos = self.registers[vx as usize] % VIDEO_WIDTH as u8;
                let y_pos = self.registers[vy as usize] % VIDEO_HEIGHT as u8;

                // println!("Drawing pixel at {},{}", x_pos, y_pos);

                self.registers[0xF] = 0;

                for row in 0..height {
                    let sprite_byte = self.memory[(self.index + row) as usize];
                    for col in 0..8 {
                        let sprite_pixel = sprite_byte & (0x80 >> col);

                        // println!("yPos + row: {}", (y_pos as u16 + row) as usize);
                        // println!("xPos + col: {}", (x_pos as u16 + col) as usize);
                        // println!("(y_pos as u16 + row) as usize * VIDEO_WIDTH: {}", (y_pos as u16 + row) as usize * VIDEO_WIDTH);

                        let idx: usize = min((y_pos as u16 + row) as usize * VIDEO_WIDTH
                            + (x_pos as u16 + col) as usize, 2047);
                        // println!("Index: {}", idx);
                        let screen_pixel = self.video[idx];
                        if sprite_pixel != 0 {
                            if screen_pixel == 0xFFFFFFFF {
                                self.registers[0xF] = 1;
                            }
                            self.video[idx] ^= 0xFFFFFFFF;
                        }
                    }
                }
            }
            Operation::SkpEx9e(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let key = self.registers[vx as usize];
                if self.keypad[key as usize] == 1 {
                    self.program_counter += 2;
                }
            }
            Operation::SknpExA1(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let key = self.registers[vx as usize];
                if self.keypad[key as usize] != 1 {
                    self.program_counter += 2;
                }
            }
            Operation::LdFx07(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;

                self.registers[vx as usize] = self.delay_timer;
            }
            Operation::LdFx0a(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;

                if self.keypad[0] == 1 {
                    self.registers[vx as usize] = 0;
                } else if self.keypad[1] == 1 {
                    self.registers[vx as usize] = 1;
                } else if self.keypad[2] == 1 {
                    self.registers[vx as usize] = 2;
                } else if self.keypad[3] == 1 {
                    self.registers[vx as usize] = 3;
                } else if self.keypad[4] == 1 {
                    self.registers[vx as usize] = 4;
                } else if self.keypad[5] == 1 {
                    self.registers[vx as usize] = 5;
                } else if self.keypad[6] == 1 {
                    self.registers[vx as usize] = 6;
                } else if self.keypad[7] == 1 {
                    self.registers[vx as usize] = 7;
                } else if self.keypad[8] == 1 {
                    self.registers[vx as usize] = 8;
                } else if self.keypad[9] == 1 {
                    self.registers[vx as usize] = 9;
                } else if self.keypad[10] == 1 {
                    self.registers[vx as usize] = 10;
                } else if self.keypad[11] == 1 {
                    self.registers[vx as usize] = 11;
                } else if self.keypad[12] == 1 {
                    self.registers[vx as usize] = 12;
                } else if self.keypad[13] == 1 {
                    self.registers[vx as usize] = 13;
                } else if self.keypad[14] == 1 {
                    self.registers[vx as usize] = 14;
                } else if self.keypad[15] == 1 {
                    self.registers[vx as usize] = 15;
                } else {
                    self.program_counter -= 2;
                }
            }
            Operation::LdFx15(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                self.delay_timer = self.registers[vx as usize];
            }
            Operation::LdFx18(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                self.sound_timer = self.registers[vx as usize];
            }
            Operation::AddFx1e(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                self.index += self.registers[vx as usize] as u16;
            }
            Operation::LdFx29(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let digit = self.registers[vx as usize];
                self.index = FONTSET_START_ADDRESS as u16 + (5 * digit) as u16;
            }
            Operation::LdFx33(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;
                let mut value = self.registers[vx as usize];

                self.memory[(self.index + 2) as usize] = value % 10;
                value /= 10;

                self.memory[(self.index + 1) as usize] = value % 10;
                value /= 10;

                self.memory[(self.index) as usize] = value % 10;
            }
            Operation::LdFx55(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;

                for i in 0..vx {
                    let idx = self.index + i as u16;
                    self.memory[idx as usize] = self.registers[i as usize];
                }
            }
            Operation::LdFx65(opcode) => {
                let vx = ((opcode & 0x0F00) >> 8) as u8;

                for i in 0..vx {
                    let idx = self.index + i as u16;
                    self.registers[i as usize] = self.memory[idx as usize];
                }
            }
            Operation::Null(_) => {}
        }
    }

    pub fn cycle(&mut self) {
        let opcode = (self.memory[self.program_counter as usize] as u16) << 8
            | self.memory[self.program_counter as usize + 1] as u16;

        // println!("OpCode: {:#?}", opcode);

        let nibbles = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );

        let op = match nibbles {
            (0x00, 0x00, 0x0e, 0x00) => Operation::Cls00E0(0),
            (0x00, 0x00, 0x0e, 0x0e) => Operation::Ret00EE(0),
            (0x01, _, _, _) => Operation::Jp1nnn(opcode),
            (0x02, _, _, _) => Operation::Call2nnn(opcode),
            (0x03, _, _, _) => Operation::Se3xkk(opcode),
            (0x04, _, _, _) => Operation::Sne4xkk(opcode),
            (0x05, _, _, 0x00) => Operation::Se5xy0(opcode),
            (0x06, _, _, _) => Operation::Ld6xkk(opcode),
            (0x07, _, _, _) => Operation::Add7xkk(opcode),
            (0x08, _, _, 0x00) => Operation::Ld8xy0(opcode),
            (0x08, _, _, 0x01) => Operation::Or8xy1(opcode),
            (0x08, _, _, 0x02) => Operation::And8xy2(opcode),
            (0x08, _, _, 0x03) => Operation::Xor8xy3(opcode),
            (0x08, _, _, 0x04) => Operation::Add8xy4(opcode),
            (0x08, _, _, 0x05) => Operation::Sub8xy5(opcode),
            (0x08, _, _, 0x06) => Operation::Shr8xy6(opcode),
            (0x08, _, _, 0x07) => Operation::Subn8xy7(opcode),
            (0x08, _, _, 0x0e) => Operation::Shl8xyE(opcode),
            (0x09, _, _, 0x00) => Operation::Sne9xy0(opcode),
            (0x0a, _, _, _) => Operation::LdAnnn(opcode),
            (0x0b, _, _, _) => Operation::JpBnnn(opcode),
            (0x0c, _, _, _) => Operation::RndCxkk(opcode),
            (0x0d, _, _, _) => Operation::DrwDxyn(opcode),
            (0x0e, _, 0x09, 0x0e) => Operation::SkpEx9e(opcode),
            (0x0e, _, 0x0a, 0x01) => Operation::SknpExA1(opcode),
            (0x0f, _, 0x00, 0x07) => Operation::LdFx07(opcode),
            (0x0f, _, 0x00, 0x0a) => Operation::LdFx0a(opcode),
            (0x0f, _, 0x01, 0x05) => Operation::LdFx15(opcode),
            (0x0f, _, 0x01, 0x08) => Operation::LdFx18(opcode),
            (0x0f, _, 0x01, 0x0e) => Operation::AddFx1e(opcode),
            (0x0f, _, 0x02, 0x09) => Operation::LdFx29(opcode),
            (0x0f, _, 0x03, 0x03) => Operation::LdFx33(opcode),
            (0x0f, _, 0x05, 0x05) => Operation::LdFx55(opcode),
            (0x0f, _, 0x06, 0x05) => Operation::LdFx65(opcode),
            _ => Operation::Null(0),
        };

        // println!("Executing Op: {:#?}", op);

        self.program_counter += 2;

        self.execute(op);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn process_input(&mut self, event_pump: &mut EventPump) -> bool {
        let mut quit = false;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    quit = true;
                }
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    println!("Keycode: {}", keycode);
                    match keycode {
                        Keycode::Escape => quit = true,
                        Keycode::X => self.keypad[0] = 1,
                        Keycode::Num1 => self.keypad[1] = 1,
                        Keycode::Num2 => self.keypad[2] = 1,
                        Keycode::Num3 => self.keypad[3] = 1,
                        Keycode::Q => self.keypad[4] = 1,
                        Keycode::W => self.keypad[5] = 1,
                        Keycode::E => self.keypad[6] = 1,
                        Keycode::A => self.keypad[7] = 1,
                        Keycode::S => self.keypad[8] = 1,
                        Keycode::D => self.keypad[9] = 1,
                        Keycode::Z => self.keypad[0xA] = 1,
                        Keycode::C => self.keypad[0xB] = 1,
                        Keycode::Num4 => self.keypad[0xC] = 1,
                        Keycode::R => self.keypad[0xD] = 1,
                        Keycode::F => self.keypad[0xE] = 1,
                        Keycode::V => self.keypad[0xF] = 1,
                        _ => continue,
                    }
                }
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    println!("Keycode: {}", keycode);
                    match keycode {
                        Keycode::X => self.keypad[0] = 0,
                        Keycode::Num1 => self.keypad[1] = 0,
                        Keycode::Num2 => self.keypad[2] = 0,
                        Keycode::Num3 => self.keypad[3] = 0,
                        Keycode::Q => self.keypad[4] = 0,
                        Keycode::W => self.keypad[5] = 0,
                        Keycode::E => self.keypad[6] = 0,
                        Keycode::A => self.keypad[7] = 0,
                        Keycode::S => self.keypad[8] = 0,
                        Keycode::D => self.keypad[9] = 0,
                        Keycode::Z => self.keypad[0xA] = 0,
                        Keycode::C => self.keypad[0xB] = 0,
                        Keycode::Num4 => self.keypad[0xC] = 0,
                        Keycode::R => self.keypad[0xD] = 0,
                        Keycode::F => self.keypad[0xE] = 0,
                        Keycode::V => self.keypad[0xF] = 0,
                        _ => continue,
                    }
                }
                _ => {}
            }
        }
        quit
    }

    pub fn keypad(&self) -> &[u8; 16] {
        &self.keypad
    }

    pub fn video(&self) -> Vec<u8> {
        let mut result = Vec::new();

        for rgba in self.video {
            result.push(((rgba >> 16) & 0xFF) as u8);
            result.push(((rgba >> 8) & 0xFF) as u8);
            result.push(((rgba) & 0xFF) as u8);
            result.push(((rgba >> 24) & 0xFF) as u8);
        }

        result
    }

    fn safe_subtract(lhs: u8, rhs: u8) -> u8 {
        ((lhs as i32 - rhs as i32) & 0x00FF) as u8
    }

    fn get_random_number() -> u8 {
        rand::thread_rng().gen_range(0..=255)
    }
}
