

use std::collections::VecDeque;

use crate::emu::{
    bios::BIOS_START,
};

use super::Psx;

struct RegisterIndex(u32);

#[derive(Debug)]
pub struct Cpu {
    /// Program counter register
    pc: u32,
    /// General purpose registers
    /// The first entry [0] must always contain 0
    regs: [u32; 32],
    delay_queue: VecDeque<Instruction>,
}

impl Cpu {
    pub fn new() -> Self {
        let mut regs = [0xdeadbeef; 32];
        regs[0] = 0;

        // Delay queue begins with 1 NOOP
        let delay_queue = VecDeque::from([Instruction(0)]);

        Cpu {
            pc: BIOS_START,
            regs,
            delay_queue,
        }
    }

    fn reg(&self, idx: RegisterIndex) -> u32 {
        let RegisterIndex(i) = idx;
        self.regs[i as usize]
    }

    fn set_reg(&mut self, idx: RegisterIndex, val: u32) {
        let RegisterIndex(i) = idx;
        self.regs[i as usize] = val;
        self.regs[0] = 0;
    }
}

pub fn handle_next_instruction(psx: &mut Psx) {

    let inst = psx.cpu.delay_queue.pop_back()
        .expect("delay queue empty. cannot fetch instruction");

    let next_inst = fetch_instruction(psx);
    psx.cpu.delay_queue.push_front(next_inst);
    psx.cpu.pc = psx.cpu.pc.wrapping_add(4);

    log::trace!("raw instruction: 0x{:08x}", inst.0);

    // Primary opcode
    match inst.opcode() {
        0x0F => op_lui(psx, inst),
        0x0D => op_ori(psx, inst),
        0x2B => op_sw(psx, inst),
        0x09 => op_addiu(psx, inst),
        0x02 => op_j(psx, inst),
        0x10 => op_cop0(psx, inst),
        0x00 => {
            // Secondary opcode
            match inst.funct() { 
                0x00 => op_sll(psx, inst),
                0x25 => op_or(psx, inst),
                unknown => panic!("unknown secondary opcode: 0x{unknown:02x}"),
            }
        }
        unknown => panic!("unknown primary opcode: 0x{unknown:02x}"),
    }

    /*
    println!("Instruction 0x{:08x} handled. New state:", inst_debug.0);
    println!("Registers: ");
    for (idx, reg) in psx.cpu.regs.iter().enumerate() {
        println!("[{idx:2}]: {reg:08x}");
    }
    */

    //log::debug!("Cpu state after instruction:\n{:#x?}", psx.cpu);
}

fn fetch_instruction(psx: &mut Psx) -> Instruction {
    let addr = psx.cpu.pc;
    Instruction(psx.load32(addr))
}

#[derive(Debug, Copy, Clone)]
pub struct Instruction(u32);

impl Instruction {

    /// Return bits [31:26] of instruction
    fn opcode(self) -> u32 {
        let Instruction(op) = self;
        op >> 26
    }

    /// Return subfunction value in bits [5:0]
    fn funct(self) -> u32 {
        let Instruction(op) = self;
        op & 0x3f
    }

    /// Return coprocessor opcode value in bits [25:21]
    fn cop_op(self) -> u32 {
        let Instruction(op) = self;
        let i = (op >> 21) & 0x1f;
        i
    }

    /// Return register index in bits [25:21]
    fn rs(self) -> RegisterIndex {
        let Instruction(op) = self;
        let i = (op >> 21) & 0x1f;
        RegisterIndex(i)
    }

    /// Return register index in bits [20:16]
    fn rt(self) -> RegisterIndex {
        let Instruction(op) = self;
        let i = (op >> 16) & 0x1f;
        RegisterIndex(i)
    }

    /// Return register index in bits [15:11]
    fn rd(self) -> RegisterIndex {
        let Instruction(op) = self;
        let i = (op >> 11) & 0x1f;
        RegisterIndex(i)
    }

    /// Return 'Shift amount immediate' value in bits [10:6]
    fn shamt(self) -> u32 {
        let Instruction(op) = self;
        (op >> 6) & 0x1f
    }

    /// Return immediate value in bits [16:0]
    fn imm(self) -> u32 {
        let Instruction(op) = self;
        op & 0xffff
    }

    /// Return immediate value in bits [16:0] as a sign-extended 32 bit value
    fn imm_se(self) -> u32 {
        let Instruction(op) = self;
        let sign_extend = |n: u32| { n as i16 as u32 };
        sign_extend(op & 0xffff)
    }

    /// Return immediate value in bits [25:0] used for jump address
    fn addr(self) -> u32 {
        let Instruction(op) = self;
        op & 0x3ff_ffff
    }
}

/// Load upper (immediate)
/// lui rt,imm
/// rt = (0x0000..0xffff) << 16
fn op_lui(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec LUI");
    let i = inst.imm();
    let rt = inst.rt(); // TODO: Pipelining

    // Low 16 bits are set to 0
    let v = i << 16;
    psx.cpu.set_reg(rt, v);
}

/// Bitwise OR
/// or rd,rs,rt
/// rd = rs OR rt
fn op_or(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec OR");
    let rd = inst.rd();
    let rs = inst.rs();
    let rt = inst.rt();

    let v = psx.cpu.reg(rs) | psx.cpu.reg(rt);
    psx.cpu.set_reg(rd, v);
}

/// Bitwise OR (immediate)
/// ori rs,rt,imm
/// rt = rs | (0x0000..0xffff)
fn op_ori(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec ORI");
    let i = inst.imm();
    let rt = inst.rt();
    let rs = inst.rs();

    let v = psx.cpu.reg(rs) | i;
    psx.cpu.set_reg(rt, v);
}

/// Store word
/// sw rt,imm(rs)
/// [imm + rs] = rt
fn op_sw(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SW");
    let i = inst.imm_se();
    let rt = inst.rt();
    let rs = inst.rs();

    let addr = psx.cpu.reg(rs).wrapping_add(i);
    let v = psx.cpu.reg(rt);

    psx.store32(addr, v);
}

/// Shift left logical
/// sll rd,rt,imm
/// rd = rt << (0x00..0x1f)
fn op_sll(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SLL");
    let i = inst.shamt();
    let rt = inst.rt();
    let rd = inst.rd();

    let v = psx.cpu.reg(rt) << i;

    psx.cpu.set_reg(rd, v);
}

/// Add Immediate Unsigned
/// addiu rt,rs,imm
/// rt = rs + (-0x8000..+0x7fff)
fn op_addiu(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec ADDIU");
    let i = inst.imm_se();
    let rt = inst.rt();
    let rs = inst.rs();

    let v = psx.cpu.reg(rs).wrapping_add(i);
    psx.cpu.set_reg(rt, v);
}

/// Jump
/// j addr
/// pc = (pc & 0xf000_0000) + (addr * 4), ra = $ + 8
fn op_j(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec J");
    let a = inst.addr();

    psx.cpu.pc = (psx.cpu.pc & 0xf000_0000) | (a << 2);
}

/* Coprocessor logic */

/// Invoke coprocessor 0
/// cop0 cop_op
/// exec cop0 command 0x0..0x1ff_ffff
fn op_cop0(psx: &mut Psx, inst: Instruction) {
    match inst.cop_op() {
        0x4 => op_mtc0(psx, inst),
        other => panic!("unknwon cop0 delegation: {other:05x}"),
    }
}

fn op_mtc0(psx: &mut Psx, inst: Instruction) { 
    todo!()
}