//noinspection SpellCheckingInspection
pub trait Decoder<T> {
    fn add(&mut self, s: u8, t: u8, d: u8) -> T;
    fn addu(&mut self, s: u8, t: u8, d: u8) -> T;
    fn and(&mut self, s: u8, t: u8, d: u8) -> T;
    fn div(&mut self, s: u8, t: u8) -> T;
    fn divu(&mut self, s: u8, t: u8) -> T;
    fn mult(&mut self, s: u8, t: u8) -> T;
    fn multu(&mut self, s: u8, t: u8) -> T;
    fn nor(&mut self, s: u8, t: u8, d: u8) -> T;
    fn or(&mut self, s: u8, t: u8, d: u8) -> T;
    fn sll(&mut self, t: u8, d: u8, sham: u8) -> T;
    fn sllv(&mut self, s: u8, t: u8, d: u8) -> T;
    fn sra(&mut self, t: u8, d: u8, sham: u8) -> T;
    fn srav(&mut self, s: u8, t: u8, d: u8) -> T;
    fn srl(&mut self, t: u8, d: u8, sham: u8) -> T;
    fn srlv(&mut self, s: u8, t: u8, d: u8) -> T;
    fn sub(&mut self, s: u8, t: u8, d: u8) -> T;
    fn subu(&mut self, s: u8, t: u8, d: u8) -> T;
    fn xor(&mut self, s: u8, t: u8, d: u8) -> T;
    fn slt(&mut self, s: u8, t: u8, d: u8) -> T;
    fn sltu(&mut self, s: u8, t: u8, d: u8) -> T;
    fn jr(&mut self, s: u8) -> T;
    fn jalr(&mut self, s: u8) -> T;

    fn addi(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn addiu(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn andi(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn ori(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn xori(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn lhi(&mut self, t: u8, imm: u16) -> T;
    fn llo(&mut self, t: u8, imm: u16) -> T;
    fn slti(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn sltiu(&mut self, s: u8, t: u8, imm: u16) -> T;

    fn beq(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn bne(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn bgtz(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn blez(&mut self, s: u8, t: u8, imm: u16) -> T;

    fn bltz(&mut self, s: u8, imm: u16) -> T;
    fn bgez(&mut self, s: u8, imm: u16) -> T;
    fn bltzal(&mut self, s: u8, imm: u16) -> T;
    fn bgezal(&mut self, s: u8, imm: u16) -> T;

    fn j(&mut self, imm: u32) -> T;
    fn jal(&mut self, imm: u32) -> T;

    fn lb(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn lbu(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn lh(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn lhu(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn lw(&mut self, s: u8, t: u8, imm: u16) -> T;

    fn sb(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn sh(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn sw(&mut self, s: u8, t: u8, imm: u16) -> T;

    fn mfhi(&mut self, d: u8) -> T;
    fn mflo(&mut self, d: u8) -> T;
    fn mthi(&mut self, s: u8) -> T;
    fn mtlo(&mut self, s: u8) -> T;

    fn trap(&mut self) -> T;

    fn dispatch_rtype(&mut self, instruction: u32) -> Option<T> {
        let func = instruction & 0x3F;

        let s = ((instruction >> 21) & 0x1F) as u8;
        let t = ((instruction >> 16) & 0x1F) as u8;
        let d = ((instruction >> 11) & 0x1F) as u8;
        let sham = ((instruction >> 6) & 0x1F) as u8;

        Some(match func {
            0 => self.sll(t, d, sham),
            2 => self.srl(t, d, sham),
            3 => self.sra(t, d, sham),
            4 => self.sllv(s, t, d),
            6 => self.srlv(s, t, d),
            7 => self.srav(s, t, d),
            8 => self.jr(s),
            9 => self.jalr(s),
            16 => self.mfhi(d),
            17 => self.mthi(s),
            18 => self.mflo(d),
            19 => self.mtlo(s),
            24 => self.mult(s, t),
            25 => self.multu(s, t),
            26 => self.div(s, t),
            27 => self.divu(s, t),
            32 => self.add(s, t, d),
            33 => self.addu(s, t, d),
            34 => self.sub(s, t, d),
            35 => self.subu(s, t, d),
            36 => self.and(s, t, d),
            37 => self.or(s, t, d),
            38 => self.xor(s, t, d),
            39 => self.nor(s, t, d),
            41 => self.sltu(s, t, d),
            42 => self.slt(s, t, d),

            _ => return None
        })
    }

    fn dispatch_special(&mut self, instruction: u32) -> Option<T> {
        let s = ((instruction >> 21) & 0x1F) as u8;
        let t = ((instruction >> 16) & 0x1F) as u8;
        let imm = (instruction & 0xFFFF) as u16;

        Some(match t {
            0 => self.bltz(s, imm),
            1 => self.bgez(s, imm),
            16 => self.bltzal(s, imm),
            17 => self.bgezal(s, imm),

            _ => return None
        })
    }

    fn dispatch(&mut self, instruction: u32) -> Option<T> {
        let opcode = instruction >> 26;

        let s = ((instruction >> 21) & 0x1F) as u8;
        let t = ((instruction >> 16) & 0x1F) as u8;
        let imm = (instruction & 0xFFFF) as u16;
        let address = instruction & 0x003FFFFF;

        Some(match opcode {
            0 => return self.dispatch_rtype(instruction),
            1 => return self.dispatch_special(instruction),
            2 => self.j(address),
            3 => self.jal(address),
            4 => self.beq(s, t, imm),
            5 => self.bne(s, t, imm),
            6 => self.blez(s, t, imm),
            7 => self.bgtz(s, t, imm),
            8 => self.addi(s, t, imm),
            9 => self.addiu(s, t, imm),
            10 => self.slti(s, t, imm),
            11 => self.sltiu(s, t, imm),
            12 => self.andi(s, t, imm),
            13 => self.ori(s, t, imm),
            14 => self.xori(s, t, imm),
            15 => self.lhi(t, imm), // # LUI
            24 => self.llo(t, imm),
            25 => self.lhi(t, imm),
            26 => self.trap(),
            32 => self.lb(s, t, imm),
            33 => self.lh(s, t, imm),
            35 => self.lw(s, t, imm),
            36 => self.lbu(s, t, imm),
            37 => self.lhu(s, t, imm),
            40 => self.sb(s, t, imm),
            41 => self.sh(s, t, imm),
            43 => self.sw(s, t, imm),

            _ => return None
        })
    }
}
