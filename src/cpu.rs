use crate::{VRAM_HEIGHT, VRAM_WIDTH};
use rand::prelude::*;
use std::time::Instant;

const RAM_SIZE: usize = 4096;
const ROM_START: usize = 0x200;

const FONTS: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

struct Timer {
    value: u8,
    start_time: Instant,
}

impl Timer {
    fn new() -> Self {
        Timer {
            value: 0,
            start_time: Instant::now(),
        }
    }

    fn get_value(&mut self) -> u8 {
        let flips = (self.start_time.elapsed().as_millis() / 15) as u8;
        if flips >= self.value {
            self.value = 0;
            0
        } else {
            self.value - flips
        }
    }

    fn set_value(&mut self, value: u8) {
        self.value = value;
        self.start_time = Instant::now();
    }
}

pub struct CPU {
    v: [u8; 16],
    pc: usize,
    stack: [u16; 16],
    sp: usize,
    i: usize,
    ram: [u8; RAM_SIZE],
    pub vram: [[u8; VRAM_WIDTH]; VRAM_HEIGHT],
    delay_timer: Timer,
    sound_timer: u8,
    pub keypad: [u8; 16],
    pub update_screen: bool,
}

impl CPU {
    pub fn new() -> Self {
        let mut ram = [0; RAM_SIZE];
        ram[0..FONTS.len()].copy_from_slice(&FONTS);
        CPU {
            v: [0; 16],
            pc: ROM_START,
            stack: [0; 16],
            sp: 0,
            i: 0,
            ram,
            vram: [[0; VRAM_WIDTH]; VRAM_HEIGHT],
            delay_timer: Timer::new(),
            sound_timer: 0,
            keypad: [0; 16],
            update_screen: true,
        }
    }

    pub fn load_rom(&mut self, data: &Vec<u8>) {
        self.ram[ROM_START..ROM_START + data.len()].copy_from_slice(data);
    }

    fn get_opcode(&self) -> usize {
        (self.ram[self.pc] as usize) << 8 | (self.ram[self.pc + 1] as usize)
    }

    fn run_opcode(&mut self, opcode: usize) {
        let nibbles = (
            ((opcode & 0xF000) >> 12),
            ((opcode & 0x0F00) >> 8),
            ((opcode & 0x00F0) >> 4),
            (opcode & 0x000F),
        );
        let (_, x, y, n) = nibbles;
        let nnn = opcode & 0x0FFF;
        let kk = opcode as u8;
        match nibbles {
            (0, 0, 0xe, 0) => self.op00e0(),
            (0, 0, 0xe, 0xe) => self.op00ee(),
            (0x1, _, _, _) => self.op1nnn(nnn),
            (0x2, _, _, _) => self.op2nnn(nnn),
            (0x3, _, _, _) => self.op3xkk(x, kk),
            (0x4, _, _, _) => self.op4xkk(x, kk),
            (0x5, _, _, 0) => self.op5xy0(x, y),
            (0x6, _, _, _) => self.op6xkk(x, kk),
            (0x7, _, _, _) => self.op7xkk(x, kk),
            (0x8, _, _, 0) => self.op8xy0(x, y),
            (0x8, _, _, 1) => self.op8xy1(x, y),
            (0x8, _, _, 2) => self.op8xy2(x, y),
            (0x8, _, _, 3) => self.op8xy3(x, y),
            (0x8, _, _, 4) => self.op8xy4(x, y),
            (0x8, _, _, 5) => self.op8xy5(x, y),
            (0x8, _, _, 6) => self.op8xy6(x, y),
            (0x8, _, _, 7) => self.op8xy7(x, y),
            (0x8, _, _, 0xe) => self.op8xye(x, y),
            (0x9, _, _, 0) => self.op9xy0(x, y),
            (0xa, _, _, _) => self.opannn(nnn),
            (0xb, _, _, _) => self.opbnnn(nnn),
            (0xc, _, _, _) => self.opcxkk(x, kk),
            (0xd, _, _, _) => self.opdxyn(x, y, n),
            (0xe, _, 9, 0xe) => self.opex9e(x),
            (0xe, _, 0xa, 1) => self.opexa1(x),
            (0xf, _, 0, 7) => self.opfx07(x),
            (0xf, _, 0, 0xa) => self.opfx0a(x),
            (0xf, _, 1, 5) => self.opfx15(x),
            (0xf, _, 1, 8) => self.opfx18(x),
            (0xf, _, 1, 0xe) => self.opfx1e(x),
            (0xf, _, 2, 9) => self.opfx29(x),
            (0xf, _, 3, 3) => self.opfx33(x),
            (0xf, _, 5, 5) => self.opfx55(x),
            (0xf, _, 6, 5) => self.opfx65(x),
            _ => println!("no match opcode {:X}", opcode),
        }
        self.next();
    }

    pub fn tick(&mut self) {
        let opcode = self.get_opcode();
        self.run_opcode(opcode)
    }

    fn next(&mut self) {
        self.pc += 2;
    }

    fn jump(&mut self, adr: usize) {
        self.pc = adr - 2;
    }

    fn op00e0(&mut self) {
        //Clear the display.
        self.vram = [[0; VRAM_WIDTH]; VRAM_HEIGHT];
        self.update_screen = true;
    }

    fn op00ee(&mut self) {
        //Return from a subroutine.
        self.jump(self.stack[self.sp] as usize);
        self.sp -= 1;
    }

    fn op1nnn(&mut self, nnn: usize) {
        //Jump to location nnn.
        self.jump(nnn)
    }

    fn op2nnn(&mut self, nnn: usize) {
        //Call subroutine at nnn.
        self.sp += 1;
        self.next();
        self.stack[self.sp] = self.pc as u16;
        self.jump(nnn);
    }

    fn op3xkk(&mut self, x: usize, kk: u8) {
        //Skip next instruction if Vx = kk.
        if self.v[x] == kk {
            self.next();
        }
    }

    fn op4xkk(&mut self, x: usize, kk: u8) {
        //Skip next instruction if Vx != kk.
        if self.v[x] != kk {
            self.next();
        }
    }

    fn op5xy0(&mut self, x: usize, y: usize) {
        //Skip next instruction if Vx = Vy.
        if self.v[x] == self.v[y] {
            self.next();
        }
    }

    fn op6xkk(&mut self, x: usize, kk: u8) {
        //Set Vx = kk.
        self.v[x] = kk;
    }

    fn op7xkk(&mut self, x: usize, kk: u8) {
        //Set Vx = Vx + kk.
        self.v[x] = self.v[x].wrapping_add(kk);
    }

    fn op8xy0(&mut self, x: usize, y: usize) {
        //Set Vx = Vy.
        self.v[x] = self.v[y];
    }

    fn op8xy1(&mut self, x: usize, y: usize) {
        //Set Vx = Vx OR Vy
        self.v[x] |= self.v[y];
    }

    fn op8xy2(&mut self, x: usize, y: usize) {
        //Set Vx = Vx AND Vy.
        self.v[x] &= self.v[y];
    }

    fn op8xy3(&mut self, x: usize, y: usize) {
        //Set Vx = Vx XOR Vy.
        self.v[x] ^= self.v[y];
    }

    fn op8xy4(&mut self, x: usize, y: usize) {
        //Set Vx = Vx + Vy, set VF = carry.
        let (sum, carry) = self.v[x].overflowing_add(self.v[y]);
        self.v[0xf] = carry as u8;
        self.v[x] = sum;
    }

    fn op8xy5(&mut self, x: usize, y: usize) {
        //Set Vx = Vx - Vy, set VF = NOT borrow.
        self.v[0xf] = (self.v[x] >= self.v[y]) as u8;
        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
    }

    fn op8xy6(&mut self, x: usize, _: usize) {
        //Set Vx = Vy SHR 1.
        //Using Vx SHR 1 because of faulty docs.
        self.v[0xf] = self.v[x] & 1;
        self.v[x] = self.v[x] >> 1;
    }

    fn op8xy7(&mut self, x: usize, y: usize) {
        //Set Vx = Vy - Vx, set VF = NOT borrow.
        self.v[0xf] = (self.v[y] >= self.v[x]) as u8;
        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
    }

    fn op8xye(&mut self, x: usize, _: usize) {
        //Set Vx = Vy SHL 1.
        //Using Vx SHL 1 because of faulty docs.
        self.v[0xf] = (self.v[x] >> 7) & 1;
        self.v[x] = self.v[x] << 1;
    }

    fn op9xy0(&mut self, x: usize, y: usize) {
        //Skip next instruction if Vx != Vy.
        if self.v[x] != self.v[y] {
            self.next();
        }
    }

    fn opannn(&mut self, nnn: usize) {
        //Set I = nnn.
        self.i = nnn;
    }

    fn opbnnn(&mut self, nnn: usize) {
        //Jump to location nnn + V0.
        self.jump(nnn + self.v[0] as usize);
    }

    fn opcxkk(&mut self, x: usize, kk: u8) {
        //Set Vx = random byte AND kk.
        let mut rng = rand::thread_rng();
        self.v[x] = rng.gen::<u8>() & kk;
    }

    fn opdxyn(&mut self, x: usize, y: usize, n: usize) {
        // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
        self.v[0xf] = 0;
        for row in 0..n {
            let y = (self.v[y] as usize + row) % VRAM_HEIGHT;
            for bit in 0..8 {
                let x = (self.v[x] as usize + bit) % VRAM_WIDTH;
                let val = ((self.ram[self.i + row] as usize >> (7 - bit)) & 1) as u8;
                self.v[0xf] |= val & self.vram[y][x];
                self.vram[y][x] ^= val;
            }
        }
        self.update_screen = true;
    }

    fn opex9e(&mut self, x: usize) {
        //Skip next instruction if key with the value of Vx is pressed.
        if self.keypad[self.v[x] as usize] == 1 {
            self.next();
        }
    }

    fn opexa1(&mut self, x: usize) {
        //Skip next instruction if key with the value of Vx is not pressed.
        if self.keypad[self.v[x] as usize] != 1 {
            self.next();
        }
    }

    fn opfx07(&mut self, x: usize) {
        //Set Vx = delay timer value.
        self.v[x] = self.delay_timer.get_value();
    }

    fn opfx0a(&mut self, x: usize) {
        //Wait for a key press, store the value of the key in Vx.
        self.pc -= 2;
        for (k, val) in self.keypad.iter().enumerate() {
            if *val == 0 {
                self.v[x] = k as u8;
                self.next();
                break;
            }
        }
    }

    fn opfx15(&mut self, x: usize) {
        //Set delay timer = Vx.
        self.delay_timer.set_value(self.v[x]);
    }

    fn opfx18(&mut self, x: usize) {
        //Set sound timer = Vx.
        self.sound_timer = self.v[x];
    }

    fn opfx1e(&mut self, x: usize) {
        //Set I = I + Vx.
        self.i += self.v[x] as usize;
    }

    fn opfx29(&mut self, x: usize) {
        //Set I = location of sprite for digit Vx.
        self.i = (self.v[x] * 5) as usize;
    }

    fn opfx33(&mut self, x: usize) {
        //Store BCD representation of Vx in memory locations I, I+1, and I+2.
        self.ram[self.i] = self.v[x] / 100;
        self.ram[self.i + 1] = (self.v[x] % 100) / 10;
        self.ram[self.i + 2] = self.v[x] % 10;
    }

    fn opfx55(&mut self, x: usize) {
        //Store registers V0 through Vx in memory starting at location I.
        self.ram[self.i..self.i + x + 1].copy_from_slice(&self.v[0..x + 1]);
    }

    fn opfx65(&mut self, x: usize) {
        //Read registers V0 through Vx from memory starting at location I.
        self.v[0..x + 1].copy_from_slice(&self.ram[self.i..self.i + x + 1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //TODO write tests whenever rust supports bigarray equality...
}
