use crate::emu::{
    bios::BIOS_START,
    mmap,
};

use super::Psx;

pub struct Cpu {
    /// Program counter register
    pc: u32,
    /// General purpose registers
    /// The first entry [0] must always contain 0
    regs: [u32; 32],
}

impl Cpu {
    pub fn new() -> Self {
        let mut regs = [0xdeadbeef; 32];
        regs[0] = 0;

        Cpu {
            pc: BIOS_START,
            regs,
        }
    }

    fn reg(&self, idx: u32) -> u32 {
        self.regs[idx as usize]
    }

    fn set_reg(&mut self, idx: u32, val: u32) {
        self.regs[idx as usize] = val;
        self.regs[0] = 0;
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
    let instruction = fetch_instruction(psx);
    let inst_debug = instruction.clone();

    match instruction.opcode() {
        0x0F => op_lui(psx, instruction),
        0x0D => op_ori(psx, instruction),
        _ => panic!("Unhandled instruction: {:x}", instruction.0),
    }

    /*
    println!("Instruction 0x{:08x} handled. New state:", inst_debug.0);
    println!("Registers: ");
    for (idx, reg) in psx.cpu.regs.iter().enumerate() {
        println!("[{idx:2}]: {reg:08x}");
    }
    */

    psx.cpu.pc = psx.cpu.pc.wrapping_add(4);
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

#[derive(Copy, Clone)]
pub struct Instruction(u32);

impl Instruction {

    /// Return bits [31:26] of instruction
    fn opcode(&self) -> u32 {
        let Instruction(op) = self;
        op >> 26
    }

    /// Return register index in bits [25:21]
    fn rs(&self) -> u32 {
        let Instruction(op) = self;
        (op >> 21) & 0x1f
    }

    /// Return register index in bits [20:16]
    fn rt(&self) -> u32 {
        let Instruction(op) = self;

        (op >> 16) & 0x1f
    }

    /// Return immediate value in bits [16:0]
    fn imm(&self) -> u32 {
        let Instruction(op) = self;

        op & 0xffff
    }
}

/// lui rt,imm
fn op_lui(psx: &mut Psx, inst: Instruction) {
    let i = inst.imm();
    let rt = inst.rt(); // TODO: Pipelining

    // Low 16 bits are set to 0
    let v = i << 16;
    psx.cpu.set_reg(rt, v);
}

/// ori rs,rt,imm
fn op_ori(psx: &mut Psx, inst: Instruction) {
    let i = inst.imm();
    let rt = inst.rt();
    let rs = inst.rs();

    let v = psx.cpu.reg(rs) | i;
    psx.cpu.set_reg(rt, v);
}