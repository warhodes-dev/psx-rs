use crate::psx::{
    ic::Interconnect,
    bios::BIOS_START,
};

pub struct Cpu {
    pc: u32,
    ic: Interconnect,
}

impl Cpu {
    pub fn new(ic: Interconnect) -> Self {
        Cpu {
            pc: BIOS_START,
            ic,
        }
    }

    pub fn run_next_instruction(&mut self) {
        let instruction = self.load32(self.pc);
        self.pc = self.pc.wrapping_add(4);
        self.decode_and_execute(instruction);
    }

    fn decode_and_execute(&mut self, instruction: u32) {
        panic!("unhandled instruction @ {:08x}", instruction);
    }

    fn load32(&self, addr: u32) -> u32 {
        self.ic.load32(addr)
    }
}

pub struct Instruction(u32);

impl Instruction {

    /// Return bits [31:26] of instruction
    fn opcode(self) -> u32 {
        let Instruction(op) = self;
        op >> 26
    }

    /// Return register index in bits [20:16]
    fn t(self) -> u32 {
        let Instruction(op) = self;

        (op >> 16) & 0x1f
    }

    /// Return immediate value in bits [16:0]
    fn imm(self) -> u32 {
        let Instruction(op) = self;

        op & 0xffff
    }
}

//fn op_lui(&mut )