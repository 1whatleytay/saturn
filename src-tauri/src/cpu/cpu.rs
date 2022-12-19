use crate::cpu::decoder::Decoder;
use crate::cpu::disassemble::Disassembler;
use crate::cpu::State;

impl State {
    fn register(&mut self, index: u8) -> &mut u32 {
        &mut self.registers[index as usize]
    }

    fn skip(&mut self, imm: u16) {
        self.pc = (self.pc as i32 + ((imm as i16 as i32) << 2)) as u32;
    }

    fn jump(&mut self, bits: u32) {
        self.pc = (self.pc & 0xFC000000) | (bits << 2);
    }

    pub fn step(&mut self) -> bool {
        let instruction = self.memory.get_u32(self.pc);

        let text = Disassembler { }.dispatch(instruction)
            .unwrap_or_else(|| format!("invalid 0x{:08x}", instruction));
        println!("0x{:08x}: {}", self.pc, text);

        self.pc += 4;

        self.dispatch(instruction).is_some()
    }

    pub fn run(&mut self) {
        while self.step() { /* do nothing */ }
    }
}

impl Decoder<()> for State {
    fn add(&mut self, s: u8, t: u8, d: u8) {
        if let Some(value) = self.register(s).checked_add(*self.register(t)) {
            *self.register(d) = value
        } else {
            self.trap()
        }
    }

    fn addu(&mut self, s: u8, t: u8, d: u8) {
        *self.register(d) = *self.register(s) + *self.register(t)
    }

    fn and(&mut self, s: u8, t: u8, d: u8) {
        *self.register(d) = *self.register(s) & *self.register(t)
    }

    fn div(&mut self, s: u8, t: u8) {
        let (a, b) = (*self.register(s) as i32, *self.register(t) as i32);
        let (lo, hi) = if b != 0 { (a / b, a % b) } else { (0i32, 0i32) };

        (self.lo, self.hi) = (lo as u32, hi as u32);
    }

    fn divu(&mut self, s: u8, t: u8) {
        let (a, b) = (*self.register(s), *self.register(t));

        (self.lo, self.hi) = if b != 0 { (a / b, a % b) } else { (0u32, 0u32) };
    }

    fn mult(&mut self, s: u8, t: u8) {
        let (a, b) = (*self.register(s) as i64, *self.register(t) as i64);
        let value = (a * b) as u64;

        (self.lo, self.hi) = ((value & 0xFFFFFFFF) as u32, (value >> 32) as u32);
    }

    fn multu(&mut self, s: u8, t: u8) {
        let (a, b) = (*self.register(s) as u64, *self.register(t) as u64);
        let value = a * b;

        (self.lo, self.hi) = ((value & 0xFFFFFFFF) as u32, (value >> 32) as u32);
    }

    fn nor(&mut self, s: u8, t: u8, d: u8) {
        *self.register(d) = !(*self.register(s) | *self.register(t))
    }

    fn or(&mut self, s: u8, t: u8, d: u8) {
        *self.register(d) = *self.register(s) | *self.register(t)
    }

    fn sll(&mut self, t: u8, d: u8, sham: u8) {
        *self.register(d) = *self.register(t) << sham
    }

    fn sllv(&mut self, s: u8, t: u8, d: u8) {
        *self.register(d) = *self.register(t) << *self.register(s)
    }

    fn sra(&mut self, t: u8, d: u8, sham: u8) {
        let source = *self.register(t) as i32;

        *self.register(d) = (source >> (sham as i32)) as u32
    }

    fn srav(&mut self, s: u8, t: u8, d: u8) {
        let source = *self.register(t) as i32;

        *self.register(d) = (source >> (*self.register(s) as i32)) as u32
    }

    fn srl(&mut self, t: u8, d: u8, sham: u8) {
        *self.register(d) = *self.register(t) >> sham
    }

    fn srlv(&mut self, s: u8, t: u8, d: u8) {
        *self.register(d) = *self.register(t) >> *self.register(s)
    }

    fn sub(&mut self, s: u8, t: u8, d: u8) {
        if let Some(value) = self.register(s).checked_sub(*self.register(t)) {
            *self.register(d) = value
        } else {
            self.trap()
        }
    }

    fn subu(&mut self, s: u8, t: u8, d: u8) {
        *self.register(d) = *self.register(s) - *self.register(t)
    }

    fn xor(&mut self, s: u8, t: u8, d: u8) {
        *self.register(d) = *self.register(s) ^ *self.register(t)
    }

    fn slt(&mut self, s: u8, t: u8, d: u8) {
        let value = (*self.register(s) as i32) < (*self.register(t) as i32);

        *self.register(d) = value as u32
    }

    fn sltu(&mut self, s: u8, t: u8, d: u8) {
        let value = *self.register(s) < *self.register(t);

        *self.register(d) = value as u32
    }

    fn jr(&mut self, s: u8) {
        self.pc = *self.register(s)
    }

    fn jalr(&mut self, s: u8) {
        *self.register(31) = self.pc;

        self.pc = *self.register(s)
    }

    fn addi(&mut self, s: u8, t: u8, imm: u16) {
        let imm = imm as i16 as i32;

        if let Some(value) = (*self.register(s) as i32).checked_add(imm) {
            *self.register(t) = value as u32
        } else {
            self.trap()
        }
    }

    fn addiu(&mut self, s: u8, t: u8, imm: u16) {
        let imm = imm as i16 as i32;

        *self.register(t) = ((*self.register(s) as i32) + imm) as u32
    }

    fn andi(&mut self, s: u8, t: u8, imm: u16) {
        *self.register(t) = *self.register(s) & (imm as u32)
    }

    fn ori(&mut self, s: u8, t: u8, imm: u16) {
        *self.register(t) = *self.register(s) | (imm as u32)
    }

    fn xori(&mut self, s: u8, t: u8, imm: u16) {
        *self.register(t) = *self.register(s) ^ (imm as u32)
    }

    fn lhi(&mut self, t: u8, imm: u16) {
        let value = (*self.register(t) & 0x0000FFFF) | ((imm as u32) << 16);

        *self.register(t) = value
    }

    fn llo(&mut self, t: u8, imm: u16) {
        let value = (*self.register(t) & 0xFFFF) | (imm as u32);

        *self.register(t) = value
    }

    fn slti(&mut self, s: u8, t: u8, imm: u16) {
        let value = (*self.register(s) as i32) < (imm as i16 as i32);

        *self.register(t) = value as u32
    }

    fn sltiu(&mut self, s: u8, t: u8, imm: u16) {
        let value = (*self.register(s) as u32) < (imm as u32);

        *self.register(t) = value as u32
    }

    fn beq(&mut self, s: u8, t: u8, imm: u16) {
        if *self.register(s) == *self.register(t) {
            self.skip(imm);
        }
    }

    fn bne(&mut self, s: u8, t: u8, imm: u16) {
        if *self.register(s) != *self.register(t) {
            self.skip(imm);
        }
    }

    fn bgtz(&mut self, s: u8, t: u8, imm: u16) {
        if (*self.register(s) as i32) > (*self.register(t) as i32) {
            self.skip(imm);
        }
    }

    fn blez(&mut self, s: u8, t: u8, imm: u16) {
        if (*self.register(s) as i32) <= (*self.register(t) as i32) {
            self.skip(imm);
        }
    }

    fn bltz(&mut self, s: u8, imm: u16) -> () {
        if (*self.register(s) as i32) < 0 {
            self.skip(imm);
        }
    }

    fn bgez(&mut self, s: u8, imm: u16) -> () {
        if (*self.register(s) as i32) >= 0 {
            self.skip(imm);
        }
    }

    fn bltzal(&mut self, s: u8, imm: u16) -> () {
        if (*self.register(s) as i32) < 0 {
            *self.register(31) = self.pc;

            self.skip(imm);
        }
    }

    fn bgezal(&mut self, s: u8, imm: u16) -> () {
        if (*self.register(s) as i32) >= 0 {
            *self.register(31) = self.pc;

            self.skip(imm);
        }
    }

    fn j(&mut self, imm: u32) {
        self.jump(imm);
    }

    fn jal(&mut self, imm: u32) {
        *self.register(31) = self.pc;

        self.jump(imm);
    }

    fn lb(&mut self, s: u8, t: u8, imm: u16) {
        let address = (*self.register(s) as i32) + (imm as i16 as i32);

        *self.register(t) = self.memory.get(address as u32) as i8 as i32 as u32;
    }

    fn lbu(&mut self, s: u8, t: u8, imm: u16) {
        let address = (*self.register(s) as i32) + (imm as i16 as i32);

        *self.register(t) = self.memory.get(address as u32) as u32;
    }

    fn lh(&mut self, s: u8, t: u8, imm: u16) {
        let address = (*self.register(s) as i32) + (imm as i16 as i32);

        *self.register(t) = self.memory.get_u16(address as u32) as i16 as i32 as u32;
    }

    fn lhu(&mut self, s: u8, t: u8, imm: u16) {
        let address = (*self.register(s) as i32) + (imm as i16 as i32);

        *self.register(t) = self.memory.get_u16(address as u32) as u32;
    }

    fn lw(&mut self, s: u8, t: u8, imm: u16) {
        let address = (*self.register(s) as i32) + (imm as i16 as i32);

        *self.register(t) = self.memory.get_u32(address as u32);
    }

    fn sb(&mut self, s: u8, t: u8, imm: u16) {
        let address = (*self.register(s) as i32) + (imm as i16 as i32);
        let value = *self.register(t) as u8;

        self.memory.set(address as u32, value);
    }

    fn sh(&mut self, s: u8, t: u8, imm: u16) {
        let address = (*self.register(s) as i32) + (imm as i16 as i32);
        let value = *self.register(t) as u16;

        self.memory.set_u16(address as u32, value);
    }

    fn sw(&mut self, s: u8, t: u8, imm: u16) {
        let address = (*self.register(s) as i32) + (imm as i16 as i32);
        let value = *self.register(t);

        self.memory.set_u32(address as u32, value);
    }

    fn mfhi(&mut self, d: u8) {
        *self.register(d) = self.hi
    }

    fn mflo(&mut self, d: u8) {
        *self.register(d) = self.lo
    }

    fn mthi(&mut self, s: u8) {
        self.hi = *self.register(s)
    }

    fn mtlo(&mut self, s: u8) {
        self.lo = *self.register(s)
    }

    fn trap(&mut self) {
        println!("Trap!");
    }
}
