use crate::emu::{
    bios::BIOS_START,
    mmap,
};

use super::Psx;

pub struct Cpu {
    pc: u32,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            pc: BIOS_START,
        }
    }

    fn decode_and_execute(&mut self, instruction: u32) {
        /* 
            match instruction.opcode() {
                0b001111 => self.op_lui(instruction),
                _        => panic!("unhandled instruction @ {:08x}", instruction),
            }
            */
            panic!("unhandled instruction @ {:08x}", instruction);

    }
}

pub fn handle_instruction(psx: &mut Psx) {
    print!("Fetching instruction @ 0x{:08x} ... ", psx.cpu.pc);
    let instruction = fetch_instruction(psx);
    println!("Done! {:08x} -- {:02x}:{:1x}:{:02x}",
        instruction.0,
        instruction.opcode(),
        instruction.t(),
        instruction.imm(),);
    /*
    let opcode_lookup = instruction.opcode();
    self.pc = self.pc.wrapping_add(4);
    decode_and_execute(instruction);
    */
}

pub fn fetch_instruction(psx: &mut Psx) -> Instruction {
    let addr = psx.cpu.pc;
    if let Some(offset) = mmap::BIOS.contains(addr) {
        let raw_instruction = psx.bios.load32(offset);
        Instruction(raw_instruction)
    } else {
        panic!("Address 0x{:08x} is not located in BIOS region", addr);
    }
}

pub struct Instruction(u32);

impl Instruction {

    /// Return bits [31:26] of instruction
    fn opcode(&self) -> u32 {
        let Instruction(op) = self;
        op >> 26
    }

    /// Return register index in bits [20:16]
    fn t(&self) -> u32 {
        let Instruction(op) = self;

        (op >> 16) & 0x1f
    }

    /// Return immediate value in bits [16:0]
    fn imm(&self) -> u32 {
        let Instruction(op) = self;

        op & 0xffff
    }
}

//fn op_lui(&mut )