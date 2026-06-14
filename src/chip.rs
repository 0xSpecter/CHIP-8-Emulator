use rand;
use std::{collections::HashMap, sync::LazyLock};

static START_ADDRESS: u16 = 0x200;
static FONT_START_ADDRESS: u16 = 0x50;
static FONT_SIZE: u8 = 80;
static FONT_SET: [u8; 80] = [
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

pub struct Chip {
    registers: [u8; 16],

    memory: [u8; 4096],

    program_counter: u16,

    stack: [u16; 16],

    stack_pointer: u8,

    delay_timer: u8,
    sound_timer: u8,

    pub input: [bool; 16],

    pub display_memory: [u32; 64 * 32],

    opcode: u16,
    index: u16,
    end: u16,
}

static OPCODE_TABLE: LazyLock<HashMap<u16, fn(&mut Chip)>> = LazyLock::new(|| {
    HashMap::from([
        // annoying cast cuz rust analyzer is an idiot sometimes
        (0x00e0, Chip::op_00e0 as fn(&mut Chip)),
        (0x00ee, Chip::op_00ee),
        (0x1000, Chip::op_1nnn),
        (0x2000, Chip::op_2nnn),
        (0x3000, Chip::op_3xkk),
        (0x4000, Chip::op_4xkk),
        (0x5000, Chip::op_5xy0),
        (0x6000, Chip::op_6xkk),
        (0x7000, Chip::op_7xkk),
        (0x8000, Chip::op_8xy0),
        (0x8001, Chip::op_8xy1),
        (0x8002, Chip::op_8xy2),
        (0x8003, Chip::op_8xy3),
        (0x8004, Chip::op_8xy4),
        (0x8005, Chip::op_8xy5),
        (0x8006, Chip::op_8xy6),
        (0x8007, Chip::op_8xy7),
        (0x800e, Chip::op_8xye),
        (0x9000, Chip::op_9xy0),
        (0xa000, Chip::op_annn),
        (0xb000, Chip::op_bnnn),
        (0xc000, Chip::op_cxkk),
        (0xd000, Chip::op_dxyn),
        (0xe001, Chip::op_exa1),
        (0xe00e, Chip::op_ex9e),
        (0xf007, Chip::op_fx07),
        (0xf00a, Chip::op_fx0a),
        (0xf015, Chip::op_fx15),
        (0xf018, Chip::op_fx18),
        (0xf01e, Chip::op_fx1e),
        (0xf029, Chip::op_fx29),
        (0xf033, Chip::op_fx33),
        (0xf055, Chip::op_fx55),
        (0xf065, Chip::op_fx65),
    ])
});

impl Chip {
    pub fn new() -> Self {
        let mut chip = Chip {
            registers: [0; 16],
            memory: [0; 4096],
            program_counter: START_ADDRESS,
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            input: [false; 16],
            display_memory: [0; 64 * 32],
            opcode: 0,
            index: 0,
            end: 0,
        };

        chip.load_fontset();

        chip
    }

    // Load a rom file into memory starting from 0x200 upto 0xfff
    pub fn load_rom_into_memory(&mut self, filename: String) {
        self.memory[(START_ADDRESS as usize)..].fill(0);
        let bytes: Vec<u8> = std::fs::read(filename).unwrap();
        self.end = START_ADDRESS + bytes.len() as u16;
        for i in 0..bytes.len() {
            self.memory[START_ADDRESS as usize + i] = bytes[i];
        }
    }

    // Load the fontset into memory stating from 0x50 upto 0x1ff
    fn load_fontset(&mut self) {
        for i in 0..FONT_SIZE as usize {
            self.memory[FONT_START_ADDRESS as usize + i] = FONT_SET[i]
        }
    }

    fn decrement(&mut self) {
        self.program_counter -= 2;
    }

    fn increment(&mut self) {
        self.program_counter += 2;
    }

    // 00E0 - CLS
    // Clear the display
    fn op_00e0(&mut self) {
        self.display_memory.fill(0);
    }

    // 00EE - RET
    // Return from a subroutine.
    fn op_00ee(&mut self) {
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize];
    }

    // 1nnn - JP addr
    // Jump to location nnn
    // The interpreter sets the program counter to nnn
    fn op_1nnn(&mut self) {
        self.program_counter = self.opcode & 0x0fff;
    }

    // 2nnn - CALL addr
    // Call subroutine at nnn.
    fn op_2nnn(&mut self) {
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = self.opcode & 0x0fff;
    }

    // 3xkk - SE Vx, byte
    // Skip next instruction if register at x equals kk
    fn op_3xkk(&mut self) {
        let vx = self.vx();
        let byte = (self.opcode & 0x00ff) as u8;

        if self.registers[vx] == byte {
            self.increment();
        }
    }

    // 4xkk - SNE Vx, byte
    // Skip next instruction if register at x does not equal kk
    fn op_4xkk(&mut self) {
        let vx = self.vx();
        let byte = (self.opcode & 0x00ff) as u8;

        if self.registers[vx] != byte {
            self.increment();
        }
    }

    // 5xy0 - SE Vx, Vy
    // Skip next instruction if register at x is equal to register at y
    fn op_5xy0(&mut self) {
        let vx = self.vx();
        let vy = self.vy();

        if self.registers[vx] == self.registers[vy] {
            self.increment();
        }
    }

    // 6xkk - LD Vx, byte
    // Set Vx = kk
    fn op_6xkk(&mut self) {
        let vx = self.vx();
        let byte = (self.opcode & 0x00ff) as u8;

        self.registers[vx] = byte;
    }

    // 7xkk - ADD Vx, byte
    // Set Vx = Vx + kk.
    fn op_7xkk(&mut self) {
        let vx = self.vx();
        let byte = (self.opcode & 0x00ff) as u8;

        self.registers[vx] = self.registers[vx].wrapping_add(byte);
    }

    // 8xy0 - LD Vx, Vy
    // Set Vx = Vy.
    fn op_8xy0(&mut self) {
        let vx = self.vx();
        let vy = self.vy();

        self.registers[vx] = self.registers[vy];
    }

    // 8xy1 - OR Vx, Vy
    // Set Vx = Vx OR Vy
    fn op_8xy1(&mut self) {
        let vx = self.vx();
        let vy = self.vy();

        self.registers[vx] |= self.registers[vy];
    }

    // 8xy2 - AND Vx, Vy
    // Set Vx = Vx AND Vy
    fn op_8xy2(&mut self) {
        let vx = self.vx();
        let vy = self.vy();

        self.registers[vx] &= self.registers[vy];
    }

    // 8xy3 - XOR Vx, Vy
    // Set Vx = Vx XOR Vy
    fn op_8xy3(&mut self) {
        let vx = self.vx();
        let vy = self.vy();

        self.registers[vx] ^= self.registers[vy];
    }

    // 8xy4 - ADD Vx, Vy
    // Set Vx = Vx + Vy, set VF = carry
    fn op_8xy4(&mut self) {
        let vx = self.vx();
        let vy = self.vy();
        let x = self.registers[vx];
        let y = self.registers[vy];

        let (result, carry) = x.overflowing_add(y);

        self.registers[0xf] = carry as u8;
        self.registers[vx] = result;
    }

    // 8xy5 - SUB Vx, Vy
    // Set Vx = Vx - Vy, set VF = NOT borrow.
    fn op_8xy5(&mut self) {
        let vx = self.vx();
        let vy = self.vy();
        let x = self.registers[vx];
        let y = self.registers[vy];

        self.registers[0xf] = (x > y) as u8;
        self.registers[vx] = x.wrapping_sub(y);
    }

    // 8xy6 - SHR Vx
    // Set Vx = Vx SHR 1
    fn op_8xy6(&mut self) {
        let vx = self.vx();

        self.registers[0xf] = self.registers[vx] & 0x1;
        self.registers[vx] >>= 1;
    }

    // 8xy7 - SUBN Vx, Vy
    // Set Vx = Vy - Vx, set VF = NOT borrow
    fn op_8xy7(&mut self) {
        let vx = self.vx();
        let vy = self.vy();
        let x = self.registers[vx];
        let y = self.registers[vy];

        self.registers[0xf] = (y > x) as u8;
        self.registers[vx] = y.wrapping_sub(x);
    }

    // 8xyE - SHL Vx {, Vy}
    // Set Vx = Vx SHL 1.
    fn op_8xye(&mut self) {
        let vx = self.vx();
        let x = self.registers[vx];

        self.registers[0xf] = (x & 0x80) >> 7;
        self.registers[vx] <<= 1;
    }

    // 9xy0 - SNE Vx, Vy
    // Skip next instruction if Vx != Vy.
    fn op_9xy0(&mut self) {
        let vx = self.vx();
        let vy = self.vy();
        let x = self.registers[vx];
        let y = self.registers[vy];

        if x != y {
            self.increment();
        }
    }

    // Annn - LD I, addr
    // Set I = nnn
    fn op_annn(&mut self) {
        self.index = self.opcode & 0x0FFF
    }

    // Bnnn - JP V0, addr
    // Jump to location nnn + V0
    fn op_bnnn(&mut self) {
        self.program_counter = self.registers[0x0] as u16 + (self.opcode & 0x0fff);
    }

    // Cxkk - RND Vx, byte
    // Set Vx = random byte AND kk
    fn op_cxkk(&mut self) {
        let vx = self.vx();
        let kk = (self.opcode & 0xff) as u8;
        self.registers[vx] = rand::random::<u8>() & kk;
    }

    // Dxyn - DRW Vx, Vy, nibble
    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
    fn op_dxyn(&mut self) {
        let vx = self.vx();
        let vy = self.vy();
        let height = self.opcode & 0x000f;

        let xpos = (self.registers[vx] % 64) as u16;
        let ypos = (self.registers[vy] % 32) as u16;

        self.registers[0xf] = 0;

        for row in 0..height {
            let sprite_byte = self.memory[(self.index + row) as usize];
            for col in 0..8u16 {
                let sprite_pixel = sprite_byte & (0x80 >> col);
                if sprite_pixel != 0 {
                    let x = ((xpos + col) % 64) as usize;
                    let y = ((ypos + row) % 32) as usize;
                    let idx = y * 64 + x;
                    if self.display_memory[idx] == 0xffffffff {
                        self.registers[0xf] = 1;
                    }
                    self.display_memory[idx] ^= 0xffffffff;
                }
            }
        }
    }

    // Ex9E - SKP Vx
    // Skip next instruction if key with the value of Vx is pressed
    fn op_ex9e(&mut self) {
        let vx = self.vx();
        let key = self.registers[vx];

        if self.input[key as usize] {
            self.increment();
        }
    }

    // ExA1 - SKNP Vx
    // Skip next instruction if key with the value of Vx is not pressed.
    fn op_exa1(&mut self) {
        let vx = self.vx();
        let key = self.registers[vx];

        if !self.input[key as usize] {
            self.increment();
        }
    }

    // Fx07 - LD Vx, DT
    // Set Vx = delay timer value
    fn op_fx07(&mut self) {
        let vx = self.vx();

        self.registers[vx] = self.delay_timer;
    }

    // Fx0A - LD Vx, K
    // Wait for a key press, store the value of the key in Vx
    fn op_fx0a(&mut self) {
        let vx = self.vx();

        for i in 0..self.input.len() {
            if self.input[i] {
                self.registers[vx] = i as u8;
                return;
            }
        }

        self.decrement();
    }

    // Fx15 - LD DT, Vx
    // Set delay timer = Vx
    fn op_fx15(&mut self) {
        let vx = self.vx();

        self.delay_timer = self.registers[vx];
    }

    // Fx18 - LD ST, Vx
    // Set sound timer = Vx
    fn op_fx18(&mut self) {
        let vx = self.vx();

        self.sound_timer = self.registers[vx];
    }

    // Fx1E - ADD I, Vx
    // Set I = I + Vx
    fn op_fx1e(&mut self) {
        let vx = self.vx();

        self.index += self.registers[vx] as u16;
    }

    // Fx29 - LD F, Vx
    // Set I = location of sprite for digit Vx
    fn op_fx29(&mut self) {
        let vx = self.vx();
        let digit = self.registers[vx] as u16;

        self.index = FONT_START_ADDRESS + digit * 5;
    }

    // Fx33 - LD B, Vx
    // Store BCD representation of Vx in memory locations I, I+1, and I+2
    fn op_fx33(&mut self) {
        let vx = self.vx();
        let x = self.registers[vx];
        let i = self.index as usize;

        self.memory[i] = x / 100;
        self.memory[i + 1] = (x / 10) % 10;
        self.memory[i + 2] = x % 10;
    }

    // Fx55 - LD [I], Vx
    // Store registers V0 through Vx in memory starting at location I.
    fn op_fx55(&mut self) {
        let vx = self.vx();

        for i in 0..=vx {
            self.memory[self.index as usize + i] = self.registers[i];
        }
    }

    // Fx65 - LD Vx, [I]
    // Read registers V0 through Vx from memory starting at location I.
    fn op_fx65(&mut self) {
        let vx = self.vx();

        for i in 0..=vx {
            self.registers[i] = self.memory[self.index as usize + i];
        }
    }

    // run matching opcode
    fn runop(&mut self) {
        let a = (self.opcode & 0xf000) >> 12;
        let hash = match a {
            0x0 => self.opcode & 0x00ff,
            0x8 | 0xe => self.opcode & 0xf00f,
            0xf => self.opcode & 0xf0ff,
            _ => self.opcode & 0xf000,
        };
        match OPCODE_TABLE.get(&hash) {
            Some(op) => op(self),
            None => eprintln!(
                "unknown opcode: {:#06x} (hash: {:#06x}) at pc {:#06x}",
                self.opcode, hash, self.program_counter
            ),
        }
    }

    fn set_opcode(&mut self) {
        self.opcode = (self.memory[self.program_counter as usize] as u16) << 8
            | self.memory[self.program_counter as usize + 1] as u16;
    }

    fn vx(&mut self) -> usize {
        ((self.opcode & 0x0f00) >> 8) as usize
    }

    fn vy(&mut self) -> usize {
        ((self.opcode & 0x00f0) >> 4) as usize
    }

    pub fn cycle(&mut self) {
        self.set_opcode();
        self.increment();
        self.runop();

        if self.program_counter >= self.end {
            eprintln!("Program did not loop");
            std::process::exit(1);
        }
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}

impl Default for Chip {
    fn default() -> Self {
        Chip::new()
    }
}
