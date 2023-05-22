use std::collections::VecDeque;

use crate::emu::{
    bios::BIOS_START,
    cop,
};

use super::Psx;

#[derive(Debug)]
pub struct Cpu {
    /// Program counter register
    pc: u32,
    /// General purpose registers
    regs: [u32; 32],
    delay_queue: VecDeque<Instruction>,
    pending_load: Option<LoadDelay>,
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
            pending_load: None,
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
        let mut pc = self.pc;

        // Compensate for hardcoded +4 in handle_instruction
        pc = pc.wrapping_add(offset).wrapping_sub(4);

        self.pc = pc;
    }

    fn inc_pc(&mut self) {
        self.pc = self.pc.wrapping_add(4);
    }

    fn dec_pc(&mut self) {
        self.pc = self.pc.wrapping_sub(4);
    }
}

pub fn handle_next_instruction(psx: &mut Psx) {
    
    // Prepare *next* (not current) instruction
    let next_inst = fetch_instruction(psx);
    psx.cpu.delay_queue.push_front(next_inst);
    psx.cpu.pc = psx.cpu.pc.wrapping_add(4);

    // Get current instruction
    let inst = psx.cpu.delay_queue.pop_back()
        .expect("delay queue empty. cannot fetch instruction");

    // Primary opcode
    match inst.opcode() {
        0x04 => op_beq(psx, inst),
        0x05 => op_bne(psx, inst),
        0x06 => op_blez(psx, inst),
        0x07 => op_bgtz(psx, inst),
        0x0F => op_lui(psx, inst),
        0x0D => op_ori(psx, inst),
        0x23 => op_lw(psx, inst),
        0x2B => op_sw(psx, inst),
        0x08 => op_addi(psx, inst),
        0x09 => op_addiu(psx, inst),
        0x02 => op_j(psx, inst),
        0x03 => op_jal(psx, inst),
        0x10 => op_cop0(psx, inst),
        0x20 => op_lb(psx, inst),
        0x28 => op_sb(psx, inst),
        0x29 => op_sh(psx, inst),
        0x0c => op_andi(psx, inst),
        0x00 => {
            // Secondary opcode
            match inst.funct() { 
                0x00 => op_sll(psx, inst),
                0x08 => op_jr(psx, inst),
                0x25 => op_or(psx, inst),
                0x2B => op_sltu(psx, inst),
                0x20 => op_add(psx, inst),
                0x21 => op_addu(psx, inst),
                0x24 => op_and(psx, inst),
                _else => panic!("unknown secondary opcode: 0x{_else:02x} (0x{:08x})", inst.0),
            }
        }
        _else => panic!("unknown primary opcode: 0x{_else:02x} (0x{:08x})", inst.0),
    }
}

fn fetch_instruction(psx: &mut Psx) -> Instruction {
    let addr = psx.cpu.pc;
    let inst_raw = psx.load(addr);
    let inst = Instruction(inst_raw);
    log::trace!("fetched instruction: 0x{inst_raw:08x}"); 
    inst
}

#[derive(Debug, Copy, Clone)]
pub struct LoadDelay {
    target_reg: RegisterIndex,
    val: u32,
    //delay_cycles: u32,
}

impl LoadDelay {
    fn new(target_reg: RegisterIndex, val: u32) -> Self {
        Self { target_reg, val }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RegisterIndex(u32);

impl From<RegisterIndex> for u32 {
    fn from(r: RegisterIndex) -> Self {
        r.0
    }
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

    fn inner(self) -> u32 {
        let Instruction(inner) = self;
        inner
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
    let v = i << 16;

    psx.cpu.handle_pending_load();
    psx.cpu.set_reg(rt, v);
}



/// Load word
// lw rt,imm(rs)
// rt = [imm + rs]
fn op_lw(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec LW");

    if psx.cop0.status().is_isolate_cache() {
        log::warn!("ignoring load while cache is isolated");
        return;
    }

    let i = inst.imm_se();
    let rt = inst.rt();
    let rs = inst.rs();

    let s = psx.cpu.reg(rs);
    let addr = s.wrapping_add(i);

    let val = psx.load(addr);

    let load = LoadDelay::new(rt, val);
    psx.cpu.chain_pending_load(load);
}

/// Load halfword
// lw rt,imm(rs)
// rt = [imm + rs]
fn op_lh(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec LH");

    if psx.cop0.status().is_isolate_cache() {
        log::warn!("ignoring load while cache is isolated");
        return;
    }

    let i = inst.imm_se();
    let rt = inst.rt();
    let rs = inst.rs();

    let s = psx.cpu.reg(rs);
    let addr = s.wrapping_add(i);

    // Cast as i16 to force sign extension
    let val = psx.load::<u16>(addr) as i16;

    let load = LoadDelay::new(rt, val as u32);
    psx.cpu.chain_pending_load(load);
}

/// Load byte 
// lw rt,imm(rs)
// rt = [imm + rs]
fn op_lb(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec LB");

    if psx.cop0.status().is_isolate_cache() {
        log::warn!("ignoring load while cache is isolated");
        return;
    }

    let i = inst.imm_se();
    let rt = inst.rt();
    let rs = inst.rs();

    let s = psx.cpu.reg(rs);
    let addr = s.wrapping_add(i);

    // Cast as i8 to force sign extension
    let val = psx.load::<u8>(addr) as i8;

    let load = LoadDelay::new(rt, val as u32);
    psx.cpu.chain_pending_load(load);
}

/// Store word
// sw rt,imm(rs)
// [imm + rs] = rt
fn op_sw(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SW");

    if psx.cop0.status().is_isolate_cache() {
        log::warn!("ignoring store while cache is isolated");
        return;
    }

    let i = inst.imm_se();
    let rt = inst.rt();
    let rs = inst.rs();

    let s = psx.cpu.reg(rs);
    let addr = s.wrapping_add(i);
    let val = psx.cpu.reg(rt);

    psx.cpu.handle_pending_load();
    psx.store(addr, val);
}

/// Store halfword
//  rt,imm(rs)
//  [imm+rs]=(rt & 0xffff)
fn op_sh(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SH");

    if psx.cop0.status().is_isolate_cache() {
        log::warn!("ignoring store while cache is isolated");
        return;
    }

    let i = inst.imm_se();
    let rt = inst.rt();
    let rs = inst.rs();

    let s = psx.cpu.reg(rs);
    let addr = s.wrapping_add(i);
    let val = psx.cpu.reg(rt) as u16;

    psx.cpu.handle_pending_load();
    psx.store(addr, val);
}

/// Store byte
// rt,imm(rs)
// [imm+rs]=(rt & 0xff)
fn op_sb(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec SB");

    if psx.cop0.status().is_isolate_cache() {
        log::warn!("ignoring store while cache is isolated");
        return;
    }

    let i = inst.imm_se();
    let rt = inst.rt();
    let rs = inst.rs();

    let s = psx.cpu.reg(rs);
    let addr = s.wrapping_add(i);
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
        .expect(&format!("ADDI: Overflow ({s} + {t})"));

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

/// Add Immediate
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

/// Add Immediate Unsigned
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

/// Jump
// j addr
// pc = (pc & 0xf000_0000) + (addr * 4)
fn op_j(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec J");
    let addr = inst.addr();

    psx.cpu.pc = (psx.cpu.pc & 0xf000_0000) | (addr << 2);

    psx.cpu.handle_pending_load();
}

/// Jump and link
// jal addr
// pc = (pc & 0xf000_0000) + (addr * 4); ra = $ + 8
fn op_jal(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec JAL");
    let addr = inst.addr();
    let ra = RegisterIndex(31);

    psx.cpu.set_reg(ra, psx.cpu.pc);

    psx.cpu.pc = (psx.cpu.pc & 0xf000_0000) | (addr << 2);

    psx.cpu.handle_pending_load();
}

/// Jump register
// jr rs
// pc = rs
fn op_jr(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec JR");
    let rs = inst.rs();
    let addr = psx.cpu.reg(rs);

    psx.cpu.pc = addr;

    psx.cpu.handle_pending_load();
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

/// Branch if greater than zero
// bgtz rs,dest
// if rs > 0, pc = $ + 4 + (-0x8000..0x7FFF) * 4
fn op_bgtz(psx: &mut Psx, inst: Instruction) {
    log::trace!("exec BGTZ");
    let i = inst.imm_se();
    let rs = inst.rs();

    let s = psx.cpu.reg(rs);

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

    let s = psx.cpu.reg(rs);

    if s <= 0 {
        psx.cpu.branch(i);
    }

    psx.cpu.handle_pending_load();
}

/// Set on less than unsigned
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