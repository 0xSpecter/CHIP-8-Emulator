use std::collections::HashMap;

// this file is compiled seperatly, its just included in main for lsp reasons
struct Compiler {
    tokens: Vec<String>,
    index: usize,
    opcodes: Vec<u16>,
    labels: HashMap<String, u16>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            tokens: vec![],
            index: 0,
            opcodes: vec![],
            labels: HashMap::new(),
        }
    }

    fn assign_labels(&mut self) {
        // kinda wacky but have to loop tokens twice to assign and replace labels
        let mut program_counter: u16 = 0x200;
        let mut index: usize = 0;
        while index < self.tokens.len() {
            let index_plus = match self.tokens[index].as_str() {
                "cls" | "ret" => 0,
                "jp" | "call" | "ldi" | "jpv" => 1,
                "se" | "sne" | "ld" | "add" | "rnd" | "sev" | "mov" | "or" | "and" | "xor"
                | "addr" | "sub" | "shr" | "subn" | "shl" | "snev" => 2,
                "skp" | "sknp" | "gdt" | "key" | "sdt" | "sst" | "addi" | "font" | "bcd"
                | "stor" | "read" => 1,
                "drw" => 3,
                label => {
                    self.labels.insert(String::from(label), program_counter);
                    index += 1;
                    // does not factor into opcode array so skip before added
                    continue;
                }
            };
            index += 1 + index_plus;
            program_counter += 2;
        }
    }

    pub fn compile(&mut self, inname: String, outname: String) {
        let code = std::fs::read_to_string(inname).unwrap();
        self.tokens = code.split_whitespace().map(|s| s.to_owned()).collect();
        self.assign_labels();
        while self.index < self.tokens.len() {
            match self.tokens[self.index].as_str() {
                // Clear the display
                "cls" => self.push(0x00e0),
                // Return from a subroutine.
                "ret" => self.push(0x00ee),
                // Jump to location nnn
                "jp" => self.combine_xnnn(0x1),
                // Call subroutine at nnn.
                "call" => self.combine_xnnn(0x2),
                // Skip next instruction if register at x equals kk
                "se" => self.combine_xnkk(0x3),
                // Skip next instruction if register at x does not equal kk
                "sne" => self.combine_xnkk(0x4),
                // Skip next instruction if register at x is equal to register at y
                "sev" => self.combine_xyzn(0x5, 0x0),
                // Set Vx = kk
                "ld" => self.combine_xnkk(0x6),
                // Set Vx = Vx + kk.
                "add" => self.combine_xnkk(0x7),
                // Set Vx = Vy.
                "mov" => self.combine_xyzn(0x8, 0x0),
                // Set Vx = Vx OR Vy
                "or" => self.combine_xyzn(0x8, 0x1),
                // Set Vx = Vx AND Vy
                "and" => self.combine_xyzn(0x8, 0x2),
                // Set Vx = Vx XOR Vy
                "xor" => self.combine_xyzn(0x8, 0x3),
                // Set Vx = Vx + Vy, set VF = carry
                "addr" => self.combine_xyzn(0x8, 0x4),
                // Set Vx = Vx - Vy, set VF = NOT borrow.
                "sub" => self.combine_xyzn(0x8, 0x5),
                // Set Vx = Vx SHR 1
                "shr" => self.combine_xyzn(0x8, 0x6),
                // Set Vx = Vy - Vx, set VF = NOT borrow
                "subn" => self.combine_xyzn(0x8, 0x7),
                // Set Vx = Vx SHL 1.
                "shl" => self.combine_xyzn(0x8, 0xe),
                // Skip next instruction if Vx != Vy.
                "snev" => self.combine_xyzn(0x9, 0x0),
                // Set I = nnn
                "ldi" => self.combine_xnnn(0xa),
                // Jump to location nnn + V0
                "jpv" => self.combine_xnnn(0xb),
                // Set Vx = random byte AND kk
                "rnd" => self.combine_xnkk(0xc),
                // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
                "drw" => self.combine_xyn(),
                // Skip next instruction if key with the value of Vx is pressed
                "skp" => self.combine_xflo(0x9e),
                // Skip next instruction if key with the value of Vx is not pressed.
                "sknp" => self.combine_xflo(0xa1),
                // Set Vx = delay timer value
                "gdt" => self.combine_xflo(0x07),
                // Wait for a key press, store the value of the key in Vx
                "key" => self.combine_xflo(0x0a),
                // Set delay timer = Vx
                "sdt" => self.combine_xflo(0x15),
                // Set sound timer = Vx
                "sst" => self.combine_xflo(0x18),
                // Set I = I + Vx
                "addi" => self.combine_xflo(0x1e),
                // Set I = location of sprite for digit Vx
                "font" => self.combine_xflo(0x29),
                // Store BCD representation of Vx in memory locations I, I+1, and I+2
                "bcd" => self.combine_xflo(0x33),
                // Store registers V0 through Vx in memory starting at location I.
                "stor" => self.combine_xflo(0x55),
                // Read registers V0 through Vx from memory starting at location I.
                "read" => self.combine_xflo(0x65),
                _ => (),
            }
            self.index += 1;
        }
        let bytes: Vec<u8> = self
            .opcodes
            .iter()
            .flat_map(|op| op.to_be_bytes().to_vec())
            .collect();
        std::fs::write(outname, &bytes).unwrap();
    }

    fn combine_xnnn(&mut self, x: u16) {
        self.push((x << 12) | self.parse(&self.tokens[self.index + 1]));
        self.index += 1;
    }

    fn combine_xnkk(&mut self, x: u16) {
        self.push(
            (x << 12)
                | (self.parse(&self.tokens[self.index + 1]) << 8)
                | self.parse(&self.tokens[self.index + 2]),
        );
        self.index += 2;
    }

    fn combine_xyzn(&mut self, x: u16, n: u16) {
        self.push(
            (x << 12)
                | (self.parse(&self.tokens[self.index + 1]) << 8)
                | (self.parse(&self.tokens[self.index + 2]) << 4)
                | n,
        );
        self.index += 2;
    }

    fn combine_xflo(&mut self, lo: u16) {
        self.push((0xf << 12) | (self.parse(&self.tokens[self.index + 1]) << 8) | lo);
        self.index += 1;
    }

    fn combine_xyn(&mut self) {
        self.push(
            (0xd << 12)
                | (self.parse(&self.tokens[self.index + 1]) << 8)
                | (self.parse(&self.tokens[self.index + 2]) << 4)
                | self.parse(&self.tokens[self.index + 3]),
        );
        self.index += 3;
    }

    fn push(&mut self, opcode: u16) {
        self.opcodes.push(opcode);
    }

    fn parse(&self, token: &String) -> u16 {
        if token.starts_with("0x") {
            u16::from_str_radix(&token[2..], 16)
        } else {
            token.parse::<u16>()
        }
        .unwrap_or_else(|_| self.labels.get(token).unwrap().clone())
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut comp: Compiler = Compiler::new();
    comp.compile(args[1].clone(), args[2].clone());
}
