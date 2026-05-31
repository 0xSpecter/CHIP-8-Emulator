struct Compiler {
    tokens: Vec<String>,
    index: usize,
    opcodes: Vec<u16>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            tokens: vec![],
            index: 0,
            opcodes: vec![],
        }
    }
    pub fn compile(&mut self, inname: String, outname: String) {
        let code = std::fs::read_to_string(inname).unwrap();
        self.tokens = code.split_whitespace().map(|s| s.to_owned()).collect();
        while self.index < self.tokens.len() {
            match self.tokens[self.index].as_str() {
                "cls" => self.push(0x00e0),
                "ret" => self.push(0x00ee),
                "jp" => self.combine_xnnn(0x1),
                "call" => self.combine_xnnn(0x2),
                "se" => self.combine_xnkk(0x3),
                "sne" => self.combine_xnkk(0x4),
                "sev" => self.combine_xyzn(0x5, 0x0),
                "ld" => self.combine_xnkk(0x6),
                "add" => self.combine_xnkk(0x7),
                "mov" => self.combine_xyzn(0x8, 0x0),
                "or" => self.combine_xyzn(0x8, 0x1),
                "and" => self.combine_xyzn(0x8, 0x2),
                "xor" => self.combine_xyzn(0x8, 0x3),
                "addr" => self.combine_xyzn(0x8, 0x4),
                "sub" => self.combine_xyzn(0x8, 0x5),
                "shr" => self.combine_xyzn(0x8, 0x6),
                "subn" => self.combine_xyzn(0x8, 0x7),
                "shl" => self.combine_xyzn(0x8, 0xe),
                "snev" => self.combine_xyzn(0x9, 0x0),
                "ldi" => self.combine_xnnn(0xa),
                "jpv" => self.combine_xnnn(0xb),
                "rnd" => self.combine_xnkk(0xc),
                "drw" => self.combine_xyn(),
                "skp" => self.combine_xflo(0x9e),
                "sknp" => self.combine_xflo(0xa1),
                "gdt" => self.combine_xflo(0x07),
                "key" => self.combine_xflo(0x0a),
                "sdt" => self.combine_xflo(0x15),
                "sst" => self.combine_xflo(0x18),
                "addi" => self.combine_xflo(0x1e),
                "font" => self.combine_xflo(0x29),
                "bcd" => self.combine_xflo(0x33),
                "stor" => self.combine_xflo(0x55),
                "read" => self.combine_xflo(0x65),
                _ => (),
            }
            self.index += 1;
        }
        let bytes: Vec<u8> = self
            .opcodes
            .iter()
            .flat_map(|op| op.to_be_bytes().into_iter())
            .collect();
        std::fs::write(outname, &bytes).unwrap();
    }

    fn combine_xnnn(&mut self, x: u16) {
        self.push((x << 12) | self.tokens[self.index + 1].parse::<u16>().unwrap());
        self.index += 1;
    }

    fn combine_xnkk(&mut self, x: u16) {
        self.push(
            (x << 12)
                | (self.tokens[self.index + 1].parse::<u16>().unwrap() << 8)
                | self.tokens[self.index + 2].parse::<u16>().unwrap(),
        );
        self.index += 2;
    }

    fn combine_xyzn(&mut self, x: u16, n: u16) {
        self.push(
            (x << 12)
                | (self.tokens[self.index + 1].parse::<u16>().unwrap() << 8)
                | (self.tokens[self.index + 2].parse::<u16>().unwrap() << 4)
                | n,
        );
        self.index += 2;
    }

    fn combine_xflo(&mut self, lo: u16) {
        self.push((0xf << 12) | (self.tokens[self.index + 1].parse::<u16>().unwrap() << 8) | lo);
        self.index += 1;
    }

    fn combine_xyn(&mut self) {
        self.push(
            (0xd << 12)
                | (self.tokens[self.index + 1].parse::<u16>().unwrap() << 8)
                | (self.tokens[self.index + 2].parse::<u16>().unwrap() << 4)
                | self.tokens[self.index + 3].parse::<u16>().unwrap(),
        );
        self.index += 3;
    }

    fn push(&mut self, opcode: u16) {
        self.opcodes.push(opcode);
    }
}

