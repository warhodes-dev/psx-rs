//! CPU module for handling all CPU instructions including the dispatch of other modules 
//! (e.g. coprocessor or GPU)

use crate::emu::{
    bios::BIOS_START,
    cpu::instruction::{Instruction, RegisterIndex}
};

use super::Psx;

mod instruction;
mod cop;
mod bus;

/// The emulated CPU state
#[derive(Debug, Default)]
pub struct Cpu {
    /// Program counter register
    pc: u32,
    /// Next program counter, represents branch delay slot
    next_pc: u32,
    /// Prev program counter, used for exception handling
    prev_pc: u32,

    /// General purpose registers
    regs: [u32; 32],

    // Quotient and Remainder registers for DIV instructions
    /// Quotient register
    lo: u32,
    /// Remainder register
    hi: u32,

    /// CPU Coprocessor #0
    cop: cop::Cop0,

    /// Could contain a pending load that has not been consumed yet 
    pending_load: Option<LoadDelay>,
}

impl Cpu {
    pub fn new() -> Self {
        let mut regs = [0xdeadbeef; 32];
        regs[0] = 0;

        let pc = BIOS_START;

        Cpu {
            pc,
            next_pc: pc.wrapping_add(4),
            prev_pc: pc,
            regs,
            ..Default::default()
        }
    }

    fn reg(&self, idx: RegisterIndex) -> u32 {
        let RegisterIndex(i) = idx;
        self.regs[i as usize]
    }

    fn set_reg(&mut self, idx: RegisterIndex, val: u32) {
        let RegisterIndex(i) = idx;
        self.regs[i as usize] = val;
        self.regs[0] = 0; // Register 0 should stay 0, even if set otherwise
    }

    /// Handle any pending LoadDelay in the load delay queue
    fn handle_pending_load(&mut self) {
        if let Some(pending_load) = self.pending_load {
            let reg = pending_load.target_reg;
            let val = pending_load.val;
            self.set_reg(reg, val);

            //TODO: Handle delay cycles... somehow

            self.pending_load = None;
        }
    }

    /// Handle a pending LoadDelay, then enqueue new LoadDelay
    fn chain_pending_load(&mut self, new_load: LoadDelay) {
        if let Some(already_pending_load) = self.pending_load 
            && already_pending_load.target_reg != new_load.target_reg 
        {
            let val = already_pending_load.val;
            let reg = already_pending_load.target_reg;
            self.set_reg(reg, val);

            //TODO: Handle delay cycles... somehow
        }
        self.pending_load = Some(new_load);
    }

    fn branch(&mut self, offset: u32) {
        // TODO: Area for improvement. Try figuring out how to remove the sub(4)
        log::trace!("cpu branching to 0x{offset:08x}");

        // PC must be aligned on 32 bits
        let offset = offset << 2;

        self.next_pc = self.pc.wrapping_add(offset);
    }

    fn increment_pc(&mut self) {
        self.prev_pc = self.pc;
        self.pc = self.next_pc;
        self.next_pc = self.next_pc.wrapping_add(4);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LoadDelay {
    pub target_reg: RegisterIndex,
    pub val: u32,
    //delay_cycles: u32,
}

impl LoadDelay {
    pub fn new(target_reg: RegisterIndex, val: u32) -> Self {
        Self { target_reg, val }
    }
}

pub fn handle_next_instruction(psx: &mut Psx) {
    /* Old */
    //let next_inst = _fetch_instruction(psx);
    //psx.cpu.delay_queue.push_front(next_inst);

    /* New */
    let inst_addr = psx.cpu.pc;
    let inst = Instruction(psx.load(inst_addr));
    log::trace!("fetched instruction: 0x{:08x} @ 0x{:08x}", inst.inner(), inst_addr); 

    psx.cpu.increment_pc();

    //IDEA: Handle pending loads here? Maybe

    /* Old */
    // Get current instruction
    //let inst = psx.cpu.delay_queue.pop_back()
        //.expect("delay queue exhausted");
    //psx.cpu.increment_pc();

    dispatch_instruction(psx, inst);
}

fn fetch_instruction(psx: &mut Psx) -> Instruction {
    let addr = psx.cpu.pc;
    let inst = Instruction(psx.load(addr));
    log::trace!("fetched instruction: 0x{:08x}", inst.inner()); 
    inst
}

pub fn dispatch_instruction(psx: &mut Psx, inst: Instruction) {
    // Primary opcode
    match inst.opcode() {
        0x01 => op_bcondz(psx, inst),
        0x02 => op_j(psx, inst),
        0x03 => op_jal(psx, inst),
        0x04 => op_beq(psx, inst),
        0x05 => op_bne(psx, inst),
        0x06 => op_blez(psx, inst),
        0x07 => op_bgtz(psx, inst),
        0x08 => op_addi(psx, inst),
        0x09 => op_addiu(psx, inst),
        0x0A => op_slti(psx, inst),
        0x0B => op_sltiu(psx, inst),
        0x0C => op_andi(psx, inst),
        0x0D => op_ori(psx, inst),
        0x0F => op_lui(psx, inst),
        0x10 => op_cop0(psx, inst),
        0x20 => op_lb(psx, inst),
        0x23 => op_lw(psx, inst),
        0x24 => op_lbu(psx, inst),
        0x28 => op_sb(psx, inst),
        0x29 => op_sh(psx, inst),
        0x2B => op_sw(psx, inst),

        // Secondary Opcodes
        0x00 => match inst.funct() { 
            0x00 => op_sll(psx, inst),
            0x02 => op_srl(psx, inst),
            0x03 => op_sra(psx, inst),
            0x08 => op_jr(psx, inst),
            0x09 => op_jalr(psx, inst),
            0x1a => op_div(psx, inst),
            0x1b => op_divu(psx, inst),
            0x10 => op_mfhi(psx, inst),
            0x12 => op_mflo(psx, inst),
            0x20 => op_add(psx, inst),
            0x21 => op_addu(psx, inst),
            0x23 => op_subu(psx, inst),
            0x24 => op_and(psx, inst),
            0x25 => op_or(psx, inst),
            0x2A => op_slt(psx, inst),
            0x2B => op_sltu(psx, inst),
            _else => panic!("unknown secondary opcode: 0x{_else:02x} (0x{:08x})\nInstructions processed: {}", inst.0, psx.instruction_cnt),
        },
        _else => panic!("unknown primary opcode: 0x{_else:02x} (0x{:08x})\nInstructions processed: {}", inst.0, psx.instruction_cnt),
    }
}

/* ========= Opcodes ========= */

/// Load upper (immediate)
// lui rt,imm
// rt = (0x0000..0xffff) << 16
fn op_lui(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec LUI");
    let i = inst.imm();
    let rt = inst.rt(); // TODO: Pipelining

    // Low 16 bits are set to 0
    let val = i << 16;

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rt, val);
}

/// Load word
// lw rt,imm(rs)
// rt = [imm + rs]
fn op_lw(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec LW");

    if psx.cpu.cop.status().is_isolate_cache() {
        log::warn!("ignoring load while cache is isolated");
        return;
    }

    let base = inst.imm_se();

    let rt = inst.rt();
    let rs = inst.rs();

    let offset = psx.cpu.reg(rs);
    let addr = base.wrapping_add(offset);

    let val = psx.load(addr);

    let load = LoadDelay::new(rt, val);
    psx.cpu.chain_pending_load(load);
}

/// Load halfword
// lh rt,imm(rs)
// rt = [imm + rs]
fn op_lh(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec LH");

    if psx.cpu.cop.status().is_isolate_cache() {
        log::warn!("ignoring load while cache is isolated");
        return;
    }

    let base = inst.imm_se();

    let rt = inst.rt();
    let rs = inst.rs();

    let offset = psx.cpu.reg(rs);
    let addr = base.wrapping_add(offset);

    // Cast as i16 to force sign extension
    let val = psx.load::<u16>(addr) as i16;

    let load = LoadDelay::new(rt, val as u32);
    psx.cpu.chain_pending_load(load);
}

/// Load byte 
// lb rt,imm(rs)
// rt = [imm + rs] (With sign extension)
fn op_lb(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec LB");

    if psx.cpu.cop.status().is_isolate_cache() {
        log::warn!("ignoring load while cache is isolated");
        return;
    }

    let base = inst.imm_se();

    let rt = inst.rt();
    let rs = inst.rs();

    let offset = psx.cpu.reg(rs);
    let addr = base.wrapping_add(offset);

    // Cast as i8 to force sign extension
    let val = psx.load::<u8>(addr) as i8;

    let load = LoadDelay::new(rt, val as u32);
    psx.cpu.chain_pending_load(load);
}

/// Load byte unsigned
// lbu rt,imm(rs)
// rt = [imm + rs]
fn op_lbu(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec LB");

    if psx.cpu.cop.status().is_isolate_cache() {
        log::warn!("ignoring load while cache is isolated");
        return;
    }

    let base = inst.imm_se();

    let rt = inst.rt();
    let rs = inst.rs();

    let offset = psx.cpu.reg(rs);
    let addr = base.wrapping_add(offset);

    let val = psx.load::<u8>(addr);

    let load = LoadDelay::new(rt, val as u32);
    psx.cpu.chain_pending_load(load);
}

/// Store word
// sw rt,imm(rs)
// [imm + rs] = rt
fn op_sw(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SW");

    if psx.cpu.cop.status().is_isolate_cache() {
        log::warn!("ignoring store while cache is isolated");
        return;
    }

    let i = inst.imm_se();
    let rt = inst.rt();
    let rs = inst.rs();

    let offset = psx.cpu.reg(rs);
    let addr = offset.wrapping_add(i);
    let val = psx.cpu.reg(rt);

    psx.cpu.handle_pending_load();
    psx.store(addr, val);
}

/// Store halfword
//  rt,imm(rs)
//  [imm+rs]=(rt & 0xffff)
fn op_sh(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SH");

    if psx.cpu.cop.status().is_isolate_cache() {
        log::warn!("ignoring store while cache is isolated");
        return;
    }

    let i = inst.imm_se();
    let rt = inst.rt();
    let rs = inst.rs();

    let offset = psx.cpu.reg(rs);
    let addr = offset.wrapping_add(i);
    let val = psx.cpu.reg(rt) as u16;

    psx.cpu.handle_pending_load();
    psx.store(addr, val);
}

/// Store byte
// rt,imm(rs)
// [imm+rs]=(rt & 0xff)
fn op_sb(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SB");

    if psx.cpu.cop.status().is_isolate_cache() {
        log::warn!("ignoring store while cache is isolated");
        return;
    }

    let i = inst.imm_se();
    let rt = inst.rt();
    let rs = inst.rs();

    let offset = psx.cpu.reg(rs);
    let addr = offset.wrapping_add(i);
    let val = psx.cpu.reg(rt) as u8;

    psx.cpu.handle_pending_load();
    psx.store(addr, val);
}

/// Shift left logical
// sll rd,rt,imm
// rd = rt << (0x00..0x1f)
fn op_sll(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SLL");
    let i = inst.shamt();
    let rt = inst.rt();
    let rd = inst.rd();

    let t = psx.cpu.reg(rt);
    let val = t << i;

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rd, val);
}

/// Shift right logical
// sll rd,rt,imm
// rd = rt >> (0x00..0x1f)
fn op_srl(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SLL");
    let i = inst.shamt();
    let rt = inst.rt();
    let rd = inst.rd();

    let t = psx.cpu.reg(rt);
    let val = t >> i;

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rd, val);
}

/// Shift right arithmetic
// sra rd,rt,imm
// rd = rt >> (0x00..0x1f)
fn op_sra(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SRA");
    let i = inst.shamt();
    let rt = inst.rt();
    let rd = inst.rd();

    let t = psx.cpu.reg(rt) as i32;
    let val = (t >> i) as u32;

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rd, val);
}

/// Add
// add rd,rs,rt
// rd = rs + rt (with overflow trap)
fn op_add(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec ADD");
    let rd = inst.rd();
    let rs = inst.rs();
    let rt = inst.rt();

    let s = psx.cpu.reg(rs);
    let t = psx.cpu.reg(rt);

    let val = s.checked_add(t)
        .expect(&format!("ADD: Overflow ({s} + {t})"));

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rd, val);
}

/// Add unsigned
// addu rd,rs,rt
// rd = rs + rt
fn op_addu(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec ADDU");
    let rd = inst.rd();
    let rs = inst.rs();
    let rt = inst.rt();

    let s = psx.cpu.reg(rs);
    let t = psx.cpu.reg(rt);
    let val = s.wrapping_add(t);

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rd, val);
}

/// Add immediate
// addi rt,rs,imm
// rt = rs + (-0x8000..+0x7fff) (with overflow trap)
//
// From Flandrin's Emulation Guide:
//  The cast to i32 is important because something like 0x4 + 0xffffffff is
//  an overflow in 32bit unsigned arithmetics. If the operands are signed however
//  it’s simply 4 + -1 and that’s obviously perfectly fine. The actual result of
//  the operation would be the same (0x00000003) but since ADDI generates an
//  exception on overflow the difference in behaviour is critical.
fn op_addi(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec ADDI");
    let i = inst.imm_se() as i32;
    let rt = inst.rt();
    let rs = inst.rs();

    let s = psx.cpu.reg(rs) as i32;

    let val = s.checked_add(i)
        .expect(&format!("ADDI: Overflow ({s} + {i})"));

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rt, val as u32);
}

/// Add immediate unsigned
// addiu rt,rs,imm
// rt = rs + (-0x8000..+0x7fff)
fn op_addiu(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec ADDIU");
    let i = inst.imm_se();
    let rt = inst.rt();
    let rs = inst.rs();

    let s = psx.cpu.reg(rs);
    let val = s.wrapping_add(i);

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rt, val);
}

/// Sub
// sub rd,rs,rt
// rd = rs + rt (with overflow trap)
fn op_sub(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SUB");
    let rd = inst.rd();
    let rs = inst.rs();
    let rt = inst.rt();

    let s = psx.cpu.reg(rs);
    let t = psx.cpu.reg(rt);

    let val = s.checked_sub(t)
        .expect(&format!("SUB: Overflow ({s} + {t})"));

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rd, val);
}

/// Sub unsigned
// subu rd,rs,rt
// rd = rs + rt
fn op_subu(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SUBU");
    let rd = inst.rd();
    let rs = inst.rs();
    let rt = inst.rt();

    let s = psx.cpu.reg(rs);
    let t = psx.cpu.reg(rt);
    let val = s.wrapping_sub(t);

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rd, val);
}

/// Divide
// div rs,rt
// lo = rs / rt, hi = rs % rt
fn op_div(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec DIV");
    let rs = inst.rs();
    let rt = inst.rt();

    let num = psx.cpu.reg(rs) as i32;
    let denom = psx.cpu.reg(rt) as i32;

    match (num, denom) {
        (n, 0) if n >= 0 => { // Special case: Divide by zero (positive)
            psx.cpu.lo = -1i32 as u32;
            psx.cpu.hi = n as u32;
        },
        (n, 0) if n < 0 => { // Special case: Divide by zero (negative)
            psx.cpu.lo = 1 as u32;
            psx.cpu.hi = n as u32;
        },
        (i32::MIN, -1) => { // Special case: i32::MIN / -1 cannot fit in i32. 
            psx.cpu.lo = i32::MIN as u32;
            psx.cpu.hi = 0;
        },
        (n, d) => {
            psx.cpu.lo = (n / d) as u32;
            psx.cpu.hi = (n % d) as u32;
        },
    }

    psx.cpu.handle_pending_load();

    //TODO: Implement stalling
}

/// Divide unsigned
// divu rs,rt
// lo = rs / rt, hi = rs % rt
fn op_divu(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec DIV");
    let rs = inst.rs();
    let rt = inst.rt();

    let num = psx.cpu.reg(rs);
    let denom = psx.cpu.reg(rt);

    match (num, denom) {
        (n, 0) => { // Special case: Divide by zero
            psx.cpu.lo = -1i32 as u32;
            psx.cpu.hi = n;
        },
        (n, d) => {
            psx.cpu.lo = n / d;
            psx.cpu.hi = n % d;
        },
    }

    psx.cpu.handle_pending_load();

    //TODO: Implement stalling
}

/// Move from LO
// mflo rd
// rd = lo
fn op_mflo(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec MFLO");
    let rd = inst.rd();
    let lo = psx.cpu.lo;

    psx.cpu.set_reg(rd, lo);

    psx.cpu.handle_pending_load();

    //TODO: Implement stalling
}

/// Move from HI
// mfhi rd
// rd = lo
fn op_mfhi(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec MFLO");
    let rd = inst.rd();
    let hi = psx.cpu.hi;

    psx.cpu.set_reg(rd, hi);

    psx.cpu.handle_pending_load();

    //TODO: Implement stalling
}

/// Jump
// j addr
// pc = (pc & 0xf000_0000) + (addr * 4)
fn op_j(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec J");
    let addr = inst.addr();

    psx.cpu.next_pc = (psx.cpu.pc & 0xf000_0000) | (addr << 2);

    psx.cpu.handle_pending_load();
}

/// Jump and link
// jal addr
// pc = (pc & 0xf000_0000) + (addr * 4); ra = $ + 8
fn op_jal(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec JAL");
    let addr = inst.addr();

    let return_addr = psx.cpu.next_pc;
    psx.cpu.next_pc = (psx.cpu.pc & 0xf000_0000) | (addr << 2);

    psx.cpu.handle_pending_load();

    psx.cpu.set_reg(RegisterIndex::RETURN, return_addr);
}

/// Jump to register
// jr rs
// pc = rs
fn op_jr(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec JR");
    let rs = inst.rs();
    let addr = psx.cpu.reg(rs);

    psx.cpu.next_pc = addr;

    psx.cpu.handle_pending_load();
}

/// Jump to register and link
// (rd,)rs(,rd)
fn op_jalr(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec JALR");
    let rs = inst.rs();
    let rd = inst.rd();

    let return_addr = psx.cpu.next_pc;
    let jump_addr = psx.cpu.reg(rs);

    psx.cpu.next_pc = jump_addr; 

    psx.cpu.handle_pending_load();

    psx.cpu.set_reg(rd, return_addr);
}

/// Branch if equal
// beq rs,rt,dest
// if rs = rt, then pc = $ + 4 + (-0x8000 + 0x7fff) * 4
fn op_beq(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec BEQ");
    let i = inst.imm_se();
    let rs = inst.rs();
    let rt = inst.rt();

    let s = psx.cpu.reg(rs);
    let t = psx.cpu.reg(rt);

    if s == t {
        psx.cpu.branch(i);
    }

    psx.cpu.handle_pending_load();
}

/// Branch if not equal
// bne rs,rt,addr
// if rs != rt, pc = $ + 4 + (-0x8000..0x7FFF) * 4
fn op_bne(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec BNE");
    let i = inst.imm_se();
    let rs = inst.rs();
    let rt = inst.rt();

    let s = psx.cpu.reg(rs);
    let t = psx.cpu.reg(rt);

    if s != t {
        psx.cpu.branch(i);
    }

    psx.cpu.handle_pending_load();
}

/// Branch (condition) zero
/// This opcode can be bltz, bgez, bltzal, or bgezal
fn op_bcondz(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec BcondZ");

    let i = inst.imm_se();
    let rs = inst.rs();

    let s = psx.cpu.reg(rs) as i32;

    let discriminant = (inst.inner() >> 16) & 0x1f;
    let is_bltz = discriminant & 0x01 == 0;
    let is_bgez = discriminant & 0x01 == 1;
    let should_link = discriminant & 0x1e == 0x80;

    if should_link {
        let return_addr = psx.cpu.pc;
        psx.cpu.set_reg(RegisterIndex::RETURN, return_addr);
    }

    if is_bltz && s < 0   
    || is_bgez && s >= 0 {
        psx.cpu.branch(i);
    }

    psx.cpu.handle_pending_load();
}

/// Branch if greater than zero
// bgtz rs,dest
// if rs > 0, pc = $ + 4 + (-0x8000..0x7FFF) * 4
fn op_bgtz(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec BGTZ");
    let i = inst.imm_se();
    let rs = inst.rs();

    let s = psx.cpu.reg(rs) as i32;

    if s > 0 {
        psx.cpu.branch(i);
    }

    psx.cpu.handle_pending_load();
}

/// Branch if less than or equal to zero
// blez rs,dest
// if rs <= 0, pc = $ + 4 + (-0x8000..0x7FFF) * 4
fn op_blez(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec BGTZ");
    let i = inst.imm_se();
    let rs = inst.rs();

    let s = psx.cpu.reg(rs) as i32;

    if s <= 0 {
        psx.cpu.branch(i);
    }

    psx.cpu.handle_pending_load();
}

/// Set on less than
// stlu rd,rs,rt
// if rs < rt (signed comparison) then rd=1 else rd=0
fn op_slt(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SLT");
    let rd = inst.rd();
    let rs = inst.rs();
    let rt = inst.rt();

    let s = psx.cpu.reg(rs) as i32;
    let t = psx.cpu.reg(rt) as i32;
    let flag = (s < t) as u32;


    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rd, flag);
}

/// Set on less than, unsigned
// stlu rd,rs,rt
// if rs < rt (unsigned comparison) then rd=1 else rd=0
fn op_sltu(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SLTU");
    let rd = inst.rd();
    let rs = inst.rs();
    let rt = inst.rt();

    let s = psx.cpu.reg(rs);
    let t = psx.cpu.reg(rt);
    let flag = (s < t) as u32;


    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rd, flag);
}

/// Set on less than immediate
// slti rt,rs,imm
// if rs < imm (sign-extended immediate) (signed comparison) then rt=1, else rt=0
fn op_slti(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SLTI");
    let i = inst.imm_se() as i32;
    let rs = inst.rs();
    let rt = inst.rt();

    let s = psx.cpu.reg(rs) as i32;
    let flag = (s < i) as u32;

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rt, flag);
}

/// Set on less than immediate, unsigned
// sltiu rt,rs,imm
// if rs < imm (sign-extended immediate) (unsigned comparison) then rt=1, else rt=0
fn op_sltiu(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SLTIU");
    let i = inst.imm_se();
    let rs = inst.rs();
    let rt = inst.rt();

    let s = psx.cpu.reg(rs);
    let flag = (s < i) as u32;

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rt, flag);
}


/// Bitwise OR
// or rd,rs,rt
// rd = rs OR rt
fn op_or(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec OR");
    let rd = inst.rd();
    let rs = inst.rs();
    let rt = inst.rt();

    let s = psx.cpu.reg(rs);
    let t = psx.cpu.reg(rt);

    let val = s | t;

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rd, val);
}

/// Bitwise OR (immediate)
// ori rs,rt,imm
// rt = rs | (0x0000..0xffff)
fn op_ori(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec ORI");
    let i = inst.imm();
    let rt = inst.rt();
    let rs = inst.rs();

    let s = psx.cpu.reg(rs);
    let val = s | i;

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rt, val);
}

/// Bitwise AND
// and rd,rs,rt
// rd = rs & rt
fn op_and(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec AND");

    let rd = inst.rd();
    let rs = inst.rs();
    let rt = inst.rt();

    let s = psx.cpu.reg(rs);
    let t = psx.cpu.reg(rt);
    let d = s & t;

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rd, d);
}

/// Bitwise AND immediate
// andi rt,rs,imm
// rt = rs & imm
fn op_andi(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec ANDI");
    
    let i = inst.imm();
    let rt = inst.rt();
    let rs = inst.rs();

    let s = psx.cpu.reg(rs); 
    let t = s & i;

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rt, t);
}

/* === Coprocessor logic === */

/// Invoke coprocessor 0
// cop0 cop_op
// exec cop0 command 0x0..0x1ff_ffff
fn op_cop0(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec COP0");
    match inst.cop_op() {
        0x00 => op_mfc0(psx, inst),
        0x04 => op_mtc0(psx, inst),
        _else => panic!("unknown cop0 delegation: {_else:05x} (op: 0x{:08x})", inst.inner()),
    }
}

/// Move from coprocessor 0
// mfc0 rt,rd
// rt = cop#_(data_reg)
fn op_mfc0(psx: &mut Psx, inst: Instruction) {
    log::trace!("delegate MFC0");
    let cpu_rt = inst.rt();
    let cop_r = inst.rd();

    let val = cop::mfc0(psx, cop_r);

    let load = LoadDelay::new(cpu_rt, val);
    psx.cpu.chain_pending_load(load)
}

/// Move to coprocessor 0
// mtc0 rt,rd
// cop#_(data_reg) = rt
fn op_mtc0(psx: &mut Psx, inst: Instruction) { 
    log::trace!("delegate MTC0");
    let cpu_rt = inst.rt();
    let cop_r = inst.rd();

    let val = psx.cpu.reg(cpu_rt);

    psx.cpu.handle_pending_load();
    cop::mtc0(psx, cop_r, val);
}