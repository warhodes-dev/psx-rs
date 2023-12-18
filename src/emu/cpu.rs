//! CPU module for handling all CPU instructions including the dispatch of other modules 
//! (e.g. coprocessor or GPU)

use crate::{emu::{
    bios::BIOS_START,
    cpu::instruction::{Instruction, RegisterIndex},
    cpu::exception::{Exception, ExceptionClass},
    bus::Bus,
}, set_log_level};

use anyhow::{anyhow,Result};

use super::Psx;

mod instruction;
mod cop;
mod exception;

#[derive(Debug, Default)]
pub struct Cpu {
    /// Program counter register
    pc: u32,
    /// Next program counter, represents branch delay slot
    next_pc: u32,
    /// Current program counter, used for exception handling
    current_pc: u32,

    /// General purpose registers
    regs: [u32; 32],
    /// (Delay slot) General purpose registers
    //  Instructions will write to these which get copied back to `regs`
    //  after decode and dispatch loop.
    out_regs: [u32; 32],

    // Quotient and Remainder registers for DIV instructions
    /// Quotient register
    lo: u32,
    /// Remainder register
    hi: u32,

    /// CPU Coprocessor #0
    cop: cop::Cop0,

    /// Could contain a pending load that has not been consumed yet 
    pending_load: Option<LoadDelay>,

    /// Does the CPU have a pending branch?
    pending_branch: bool,
    /// Is the CPU in the branch delay slot?
    branch_delay_slot: bool,
}

impl Cpu {
    pub fn new() -> Self {
        let mut regs = [0xdeadbeef; 32];
        regs[0] = 0;

        let initial_pc = BIOS_START;

        Cpu {
            pc:         initial_pc,
            next_pc:    initial_pc.wrapping_add(4),
            current_pc: initial_pc,
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
        self.out_regs[i as usize] = val;
        self.out_regs[0] = 0; // Register 0 should stay 0, even if set otherwise
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
            let reg = already_pending_load.target_reg;
            let val = already_pending_load.val;
            self.set_reg(reg, val);

            //TODO: Handle delay cycles... somehow
        }
        self.pending_load = Some(new_load);
    }

    fn branch(&mut self, offset: u32) {
        tracing::trace!("cpu branching to 0x{offset:08x}");

        // PC must be aligned on 32 bits
        let offset = offset << 2;

        self.next_pc = self.pc.wrapping_add(offset);
    }

    fn exception(&mut self, cause: Exception) {
        let handler = exception::handler(
            self.cop.status().exception_vector(), 
            ExceptionClass::General,
        );

        tracing::debug!("Exception handler: 0x{handler:08x} (from BEV = {})", 
            self.cop.status().exception_vector() as u32);

        // From emulation guide, pg. 66:
        //
        // Entering an exception pushes a pair of zeroes
        // [onto the interrupt mode] stack which disables
        // interrupts and puts the CPU in kernel mode.
        self.cop.push_mode();

        self.cop.set_cause(cause);
        
        self.cop.epc = self.current_pc; //TODO: Is this right?

        self.pc = handler;
        self.next_pc = handler.wrapping_add(4);
    }

    fn increment_pc(&mut self) {
        self.pc = self.next_pc;
        self.next_pc = self.next_pc.wrapping_add(4);
    }
}

#[derive(Debug)]
enum BranchDelay {
    Branch,
    Delay,
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

impl Cpu {
    pub fn handle_next_instruction(&mut self, bus: &mut Bus) {

        // Track instruction address in case of exception
        self.current_pc = self.pc;

        if self.current_pc % 4 != 0 {
            self.exception(Exception::LoadAlignmentError);
            return;
        }

        let inst_addr = self.pc;
        let inst = Instruction(bus.load(inst_addr));

        tracing::trace!("fetched instruction: 0x{inst:08x} @ 0x{inst_addr:08x}"); 

        self.increment_pc();
        self.handle_pending_load();

        self.branch_delay_slot = self.pending_branch;
        self.pending_branch = false;

        self.dispatch_instruction(bus, inst);

        self.regs = self.out_regs;
    }

    pub fn dispatch_instruction(&mut self, bus: &mut Bus, inst: Instruction) {
        // Primary opcode
        match inst.opcode() {
            0x01 => self.op_bcondz(inst),
            0x02 => self.op_j(inst),
            0x03 => self.op_jal(inst),
            0x04 => self.op_beq(inst),
            0x05 => self.op_bne(inst),
            0x06 => self.op_blez(inst),
            0x07 => self.op_bgtz(inst),
            0x08 => self.op_addi(inst),
            0x09 => self.op_addiu(inst),
            0x0A => self.op_slti(inst),
            0x0B => self.op_sltiu(inst),
            0x0C => self.op_andi(inst),
            0x0D => self.op_ori(inst),
            0x0F => self.op_lui(inst),
            0x10 => self.op_cop0(bus, inst),
            0x20 => self.op_lb(bus, inst),
            0x21 => self.op_lh(bus, inst),
            0x23 => self.op_lw(bus, inst),
            0x24 => self.op_lbu(bus, inst),
            0x25 => self.op_lhu(bus, inst),
            0x28 => self.op_sb(bus, inst),
            0x29 => self.op_sh(bus, inst),
            0x2B => self.op_sw(bus, inst),

            // Secondary Opcodes
            0x00 => match inst.funct() { 
                0x00 => self.op_sll(inst),
                0x02 => self.op_srl(inst),
                0x03 => self.op_sra(inst),
                0x04 => self.op_sllv(inst),
                0x06 => self.op_srlv(inst),
                0x07 => self.op_srav(inst),
                0x08 => self.op_jr(inst),
                0x09 => self.op_jalr(inst),
                0x0c => self.op_syscall(inst),
                0x1a => self.op_div(inst),
                0x1b => self.op_divu(inst),
                0x10 => self.op_mfhi(inst),
                0x11 => self.op_mthi(inst),
                0x12 => self.op_mflo(inst),
                0x13 => self.op_mtlo(inst),
                0x20 => self.op_add(inst),
                0x21 => self.op_addu(inst),
                0x23 => self.op_subu(inst),
                0x24 => self.op_and(inst),
                0x25 => self.op_or(inst),
                0x27 => self.op_nor(inst),
                0x2A => self.op_slt(inst),
                0x2B => self.op_sltu(inst),
                _else => panic!("unknown secondary opcode: 0x{_else:02x} (0x{:08x})", inst.0),
            },
            _else => panic!("unknown primary opcode: 0x{_else:02x} (0x{:08x})", inst.0),
        };
    }

    /* ========= Opcodes ========= */

    /// Load upper (immediate)
    // lui rt,imm
    // rt = (0x0000..0xffff) << 16
    fn op_lui(&mut self, inst: Instruction) {
        tracing::trace!("exec LUI");
        let i = inst.imm();
        let rt = inst.rt(); // TODO: Pipelining

        // Low 16 bits are set to 0
        let val = i << 16;

        //self.handle_pending_load();
        self.set_reg(rt, val);
    }

    /// Load word
    // lw rt,imm(rs)
    // rt = [imm + rs]
    fn op_lw(&mut self, bus: &mut Bus, inst: Instruction) {
        tracing::trace!("exec LW");

        if self.cop.status().is_isolate_cache() {
            tracing::warn!("ignoring load while cache is isolated");
            return;
        }

        let base = inst.imm_se();

        let rt = inst.rt();
        let rs = inst.rs();

        let offset = self.reg(rs);
        let addr = base.wrapping_add(offset);

        if addr % 4 != 0 {
            self.exception(Exception::LoadAlignmentError)
        } else {
            let val = bus.load(addr);
            let load = LoadDelay::new(rt, val);
            self.chain_pending_load(load);
        }
    }

    /// Load halfword
    // lh rt,imm(rs)
    // rt = [imm + rs]
    fn op_lh(&mut self, bus: &mut Bus, inst: Instruction) {
        tracing::trace!("exec LH");

        if self.cop.status().is_isolate_cache() {
            tracing::warn!("ignoring load while cache is isolated");
            return;
        }

        let base = inst.imm_se();

        let rt = inst.rt();
        let rs = inst.rs();

        let offset = self.reg(rs);
        let addr = base.wrapping_add(offset);

        if addr % 2 != 0 {
            self.exception(Exception::LoadAlignmentError)
        } else {
            // Cast as i16 to force sign extension
            let val = bus.load::<u16>(addr) as i16;
            let load = LoadDelay::new(rt, val as u32);
            self.chain_pending_load(load);
        }
    }

    /// Load byte 
    // lb rt,imm(rs)
    // rt = [imm + rs] (With sign extension)
    fn op_lb(&mut self, bus: &mut Bus, inst: Instruction) {
        tracing::trace!("exec LB");

        if self.cop.status().is_isolate_cache() {
            tracing::warn!("ignoring load while cache is isolated");
            return;
        }

        let base = inst.imm_se();

        let rt = inst.rt();
        let rs = inst.rs();

        let offset = self.reg(rs);
        let addr = base.wrapping_add(offset);

        // Cast as i8 to force sign extension
        let val = bus.load::<u8>(addr) as i8;

        let load = LoadDelay::new(rt, val as u32);
        self.chain_pending_load(load);
    }

    /// Load byte unsigned
    // lbu rt,imm(rs)
    // rt = [imm + rs]
    fn op_lbu(&mut self, bus: &mut Bus, inst: Instruction) {
        tracing::trace!("exec LBU");

        if self.cop.status().is_isolate_cache() {
            tracing::warn!("ignoring load while cache is isolated");
            return;
        }

        let base = inst.imm_se();

        let rt = inst.rt();
        let rs = inst.rs();

        let offset = self.reg(rs);
        let addr = base.wrapping_add(offset);

        let val = bus.load::<u8>(addr);

        let load = LoadDelay::new(rt, val as u32);
        self.chain_pending_load(load);
    }

    /// Load halfword unsigned
    // lhu rt,imm(rs)
    // rt = [imm + rs]
    fn op_lhu(&mut self, bus: &mut Bus, inst: Instruction) {
        tracing::trace!("exec LHU");

        if self.cop.status().is_isolate_cache() {
            tracing::warn!("ignoring load while cache is isolated");
            return;
        }

        let base = inst.imm_se();

        let rt = inst.rt();
        let rs = inst.rs();

        let offset = self.reg(rs);
        let addr = base.wrapping_add(offset);

        if addr % 2 != 0 {
            self.exception(Exception::LoadAlignmentError)
        } else {
            let val = bus.load::<u16>(addr);
            let load = LoadDelay::new(rt, val as u32);
            self.chain_pending_load(load);
        }
    }

    /// Store word
    // sw rt,imm(rs)
    // [imm + rs] = rt
    fn op_sw(&mut self, bus: &mut Bus, inst: Instruction) {
        tracing::trace!("exec SW");

        if self.cop.status().is_isolate_cache() {
            tracing::warn!("ignoring store while cache is isolated");
            return;
        }

        let i = inst.imm_se();
        let rt = inst.rt();
        let rs = inst.rs();

        let offset = self.reg(rs);
        let addr = offset.wrapping_add(i);

        if addr % 4 != 0 {
            self.exception(Exception::StoreAlignmentError)
        } else {
            let val = self.reg(rt);
            bus.store(addr, val);
        }
    }

    /// Store halfword
    //  rt,imm(rs)
    //  [imm+rs]=(rt & 0xffff)
    fn op_sh(&mut self, bus: &mut Bus, inst: Instruction) {
        tracing::trace!("exec SH");

        if self.cop.status().is_isolate_cache() {
            tracing::warn!("ignoring store while cache is isolated");
            return;
        }

        let i = inst.imm_se();
        let rt = inst.rt();
        let rs = inst.rs();

        let offset = self.reg(rs);
        let addr = offset.wrapping_add(i);

        if addr % 2 != 0 {
            self.exception(Exception::StoreAlignmentError)
        } else {
            let val = self.reg(rt) as u16;
            bus.store(addr, val);
        }
    }

    /// Store byte
    // rt,imm(rs)
    // [imm+rs]=(rt & 0xff)
    fn op_sb(&mut self, bus: &mut Bus, inst: Instruction) {
        tracing::trace!("exec SB");

        if self.cop.status().is_isolate_cache() {
            tracing::warn!("ignoring store while cache is isolated");
            return;
        }

        let i = inst.imm_se();
        let rt = inst.rt();
        let rs = inst.rs();

        let offset = self.reg(rs);
        let addr = offset.wrapping_add(i);
        let val = self.reg(rt) as u8;

        //self.handle_pending_load();
        bus.store(addr, val);
    }

    /// Shift left logical
    // sll rd,rt,imm
    // rd = rt << (0x00..0x1f)
    fn op_sll(&mut self, inst: Instruction) {
        tracing::trace!("exec SLL");
        let i = inst.shamt();
        let rt = inst.rt();
        let rd = inst.rd();

        let t = self.reg(rt);
        let val = t << i;

        //self.handle_pending_load();
        self.set_reg(rd, val);
    }

    /// Shift right logical
    // sll rd,rt,imm
    // rd = rt >> (0x00..0x1f)
    fn op_srl(&mut self, inst: Instruction) {
        tracing::trace!("exec SLL");
        let i = inst.shamt();
        let rt = inst.rt();
        let rd = inst.rd();

        let t = self.reg(rt);
        let val = t >> i;

        //self.handle_pending_load();
        self.set_reg(rd, val);
    }

    /// Shift left logical variable
    // sllv rd,rt,rs
    // rd = rt << (rs & 0x1f)
    fn op_sllv(&mut self, inst: Instruction) {
        tracing::trace!("exec SLLV");
        let rt = inst.rt();
        let rs = inst.rs();
        let rd = inst.rd();

        let t = self.reg(rt);
        let s = self.reg(rs);
        let val = t << (s & 0x1f);

        self.set_reg(rd, val);
    }

    /// Shift right logical variable
    // srlv rd,rt,rs
    // rd = rt << (rs & 0x1f)
    fn op_srlv(&mut self, inst: Instruction) {
        tracing::trace!("exec SLLV");
        let rd = inst.rd();
        let rt = inst.rt();
        let rs = inst.rs();

        let t = self.reg(rt);
        let s = self.reg(rs);
        let val = t >> (s & 0x1f);

        self.set_reg(rd, val);
    }

    /// Shift right arithmetic
    // sra rd,rt,imm
    // rd = rt >> (0x00..0x1f)
    fn op_sra(&mut self, inst: Instruction) {
        tracing::trace!("exec SRA");
        let i = inst.shamt();
        let rt = inst.rt();
        let rd = inst.rd();

        let t = self.reg(rt) as i32;
        let val = (t >> i) as u32;

        //self.handle_pending_load();
        self.set_reg(rd, val);
    }

    /// Shift right arithmetic variable
    // srav rd,rt,rs
    // rd = rt >> (rs & 0x1f)
    fn op_srav(&mut self, inst: Instruction) {
        tracing::trace!("exec SRA");
        let rd = inst.rd();
        let rt = inst.rt();
        let rs = inst.rs();

        let t = self.reg(rt) as i32;
        let s = self.reg(rs);
        let val = (t >> (s & 0x1f)) as u32;

        //self.handle_pending_load();
        self.set_reg(rd, val);
    }

    /// Add
    // add rd,rs,rt
    // rd = rs + rt (with overflow trap)
    fn op_add(&mut self, inst: Instruction) {
        tracing::trace!("exec ADD");
        let rd = inst.rd();
        let rs = inst.rs();
        let rt = inst.rt();

        let s = self.reg(rs);
        let t = self.reg(rt);

        if let Some(val) = s.checked_add(t) {
            self.set_reg(rd, val);
        } else {
            self.exception(Exception::Overflow);
        }
    }

    /// Add unsigned
    // addu rd,rs,rt
    // rd = rs + rt
    fn op_addu(&mut self, inst: Instruction) {
        tracing::trace!("exec ADDU");
        let rd = inst.rd();
        let rs = inst.rs();
        let rt = inst.rt();

        let s = self.reg(rs);
        let t = self.reg(rt);
        let val = s.wrapping_add(t);

        self.set_reg(rd, val);
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
    fn op_addi(&mut self, inst: Instruction) {
        tracing::trace!("exec ADDI");
        let i = inst.imm_se() as i32;
        let rt = inst.rt();
        let rs = inst.rs();

        let s = self.reg(rs) as i32;

        if let Some(val) = s.checked_add(i) {
            self.set_reg(rt, val as u32);
        } else {
            self.exception(Exception::Overflow);
        }
    }

    /// Add immediate unsigned
    // addiu rt,rs,imm
    // rt = rs + (-0x8000..+0x7fff)
    fn op_addiu(&mut self, inst: Instruction) {
        tracing::trace!("exec ADDIU");
        let i = inst.imm_se();
        let rt = inst.rt();
        let rs = inst.rs();

        let s = self.reg(rs);
        let val = s.wrapping_add(i);

        //self.handle_pending_load();
        self.set_reg(rt, val);
    }

    /// Sub
    // sub rd,rs,rt
    // rd = rs + rt (with overflow trap)
    fn op_sub(&mut self, inst: Instruction) {
        tracing::trace!("exec SUB");
        let rd = inst.rd();
        let rs = inst.rs();
        let rt = inst.rt();

        let s = self.reg(rs);
        let t = self.reg(rt);

        let val = s.checked_sub(t)
            .expect(&format!("SUB: Overflow ({s} + {t})"));

        //self.handle_pending_load();
        self.set_reg(rd, val);
    }

    /// Sub unsigned
    // subu rd,rs,rt
    // rd = rs + rt
    fn op_subu(&mut self, inst: Instruction) {
        tracing::trace!("exec SUBU");
        let rd = inst.rd();
        let rs = inst.rs();
        let rt = inst.rt();

        let s = self.reg(rs);
        let t = self.reg(rt);
        let val = s.wrapping_sub(t);

        //self.handle_pending_load();
        self.set_reg(rd, val);
    }

    /// Divide
    // div rs,rt
    // lo = rs / rt, hi = rs % rt
    fn op_div(&mut self, inst: Instruction) {
        tracing::trace!("exec DIV");
        let rs = inst.rs();
        let rt = inst.rt();

        let num = self.reg(rs) as i32;
        let denom = self.reg(rt) as i32;

        match (num, denom) {
            (n, 0) if n >= 0 => { // Special case: Divide by zero (positive)
                self.lo = -1i32 as u32;
                self.hi = n as u32;
            },
            (n, 0) if n < 0 => { // Special case: Divide by zero (negative)
                self.lo = 1 as u32;
                self.hi = n as u32;
            },
            (i32::MIN, -1) => { // Special case: i32::MIN / -1 cannot fit in i32. 
                self.lo = i32::MIN as u32;
                self.hi = 0;
            },
            (n, d) => {
                self.lo = (n / d) as u32;
                self.hi = (n % d) as u32;
            },
        }

        //self.handle_pending_load();

        //TODO: Implement stalling
    }

    /// Divide unsigned
    // divu rs,rt
    // lo = rs / rt, hi = rs % rt
    fn op_divu(&mut self, inst: Instruction) {
        tracing::trace!("exec DIV");
        let rs = inst.rs();
        let rt = inst.rt();

        let num = self.reg(rs);
        let denom = self.reg(rt);

        match (num, denom) {
            (n, 0) => { // Special case: Divide by zero
                self.lo = -1i32 as u32;
                self.hi = n;
            },
            (n, d) => {
                self.lo = n / d;
                self.hi = n % d;
            },
        }

        //self.handle_pending_load();

        //TODO: Implement stalling
    }

    /// Move from LO
    // mflo rd
    // rd = lo
    fn op_mflo(&mut self, inst: Instruction) {
        tracing::trace!("exec MFLO");
        let rd = inst.rd();
        let lo = self.lo;

        self.set_reg(rd, lo);

        //self.handle_pending_load();

        //TODO: Implement stalling
    }

    /// Move to LO
    // mtlo rs
    // lo = rs
    fn op_mtlo(&mut self, inst: Instruction) {
        tracing::trace!("exec MTLO");
        let rs = inst.rs();

        let s = self.reg(rs);
        self.lo = s;

        //self.handle_pending_load();

        //TODO: Implement stalling
    }

    /// Move from HI
    // mfhi rd
    // rd = lo
    fn op_mfhi(&mut self, inst: Instruction) {
        tracing::trace!("exec MFLO");
        let rd = inst.rd();
        let hi = self.hi;

        self.set_reg(rd, hi);

        //self.handle_pending_load();

        //TODO: Implement stalling
    }

    /// Move to HI
    // mthi rs
    // hi = rs
    fn op_mthi(&mut self, inst: Instruction) {
        tracing::trace!("exec MTHI");
        let rs = inst.rs();

        let s = self.reg(rs);
        self.hi = s;

        //self.handle_pending_hiad();

        //TODO: Implement stalling
    }

    /// Jump
    // j addr
    // pc = (pc & 0xf000_0000) + (addr * 4)
    fn op_j(&mut self, inst: Instruction) {
        tracing::trace!("exec J");
        let addr = inst.addr();

        self.next_pc = (self.pc & 0xf000_0000) | (addr << 2);

        //self.handle_pending_load();
        self.pending_branch = true;
    }

    /// Jump and link
    // jal addr
    // pc = (pc & 0xf000_0000) + (addr * 4); ra = $ + 8
    fn op_jal(&mut self, inst: Instruction) {
        tracing::trace!("exec JAL");
        let addr = inst.addr();

        let return_addr = self.next_pc;
        self.next_pc = (self.pc & 0xf000_0000) | (addr << 2);

        //self.handle_pending_load();

        self.set_reg(RegisterIndex::RETURN, return_addr);

        self.pending_branch = true;
    }

    /// Jump to register
    // jr rs
    // pc = rs
    fn op_jr(&mut self, inst: Instruction) {
        tracing::trace!("exec JR");
        let rs = inst.rs();
        let addr = self.reg(rs);

        self.next_pc = addr;

        //self.handle_pending_load();
        self.pending_branch = true;
    }

    /// Jump to register and link
    // (rd,)rs(,rd)
    fn op_jalr(&mut self, inst: Instruction) {
        tracing::trace!("exec JALR");
        let rs = inst.rs();
        let rd = inst.rd();

        let return_addr = self.next_pc;
        let jump_addr = self.reg(rs);

        self.next_pc = jump_addr; 

        //self.handle_pending_load();

        self.set_reg(rd, return_addr);

        self.pending_branch = true;
    }

    /// Branch if equal
    // beq rs,rt,dest
    // if rs = rt, then pc = $ + 4 + (-0x8000 + 0x7fff) * 4
    fn op_beq(&mut self, inst: Instruction) {
        tracing::trace!("exec BEQ");
        let i = inst.imm_se();
        let rs = inst.rs();
        let rt = inst.rt();

        let s = self.reg(rs);
        let t = self.reg(rt);

        if s == t {
            self.branch(i);
        }

        //self.handle_pending_load();
        self.pending_branch = true;
    }

    /// Branch if not equal
    // bne rs,rt,addr
    // if rs != rt, pc = $ + 4 + (-0x8000..0x7FFF) * 4
    fn op_bne(&mut self, inst: Instruction) {
        tracing::trace!("exec BNE");
        let i = inst.imm_se();
        let rs = inst.rs();
        let rt = inst.rt();

        let s = self.reg(rs);
        let t = self.reg(rt);

        if s != t {
            self.branch(i);
        }

        //self.handle_pending_load();
        self.pending_branch = true;
    }

    /// Branch (condition) zero
    /// This opcode can be bltz, bgez, bltzal, or bgezal
    fn op_bcondz(&mut self, inst: Instruction) {
        enum BranchCondition {
            LessThan,
            GreaterEqual,
        }

        tracing::trace!("exec BcondZ");

        let i = inst.imm_se();
        let rs = inst.rs();

        let s = self.reg(rs) as i32;

        let discriminant = (inst.inner() >> 16) & 0x1f;
        let should_link = discriminant & 0x1e == 0x80;

        if should_link {
            let return_addr = self.pc;
            self.set_reg(RegisterIndex::RETURN, return_addr);
        }

        if discriminant & 1 == 0 && s < 0
        || discriminant & 1 == 1 && s >= 0 {
            self.branch(i);
        }

        //self.handle_pending_load();
        self.pending_branch = true;
    }

    /// Branch if greater than zero
    // bgtz rs,dest
    // if rs > 0, pc = $ + 4 + (-0x8000..0x7FFF) * 4
    fn op_bgtz(&mut self, inst: Instruction) {
        tracing::trace!("exec BGTZ");
        let i = inst.imm_se();
        let rs = inst.rs();

        let s = self.reg(rs) as i32;

        if s > 0 {
            self.branch(i);
        }

        //self.handle_pending_load();
    }

    /// Branch if less than or equal to zero
    // blez rs,dest
    // if rs <= 0, pc = $ + 4 + (-0x8000..0x7FFF) * 4
    fn op_blez(&mut self, inst: Instruction) {
        tracing::trace!("exec BGTZ");
        let i = inst.imm_se();
        let rs = inst.rs();

        let s = self.reg(rs) as i32;

        if s <= 0 {
            self.branch(i);
        }

        //self.handle_pending_load();
    }

    /// Set on less than
    // stlu rd,rs,rt
    // if rs < rt (signed comparison) then rd=1 else rd=0
    fn op_slt(&mut self, inst: Instruction) {
        tracing::trace!("exec SLT");
        let rd = inst.rd();
        let rs = inst.rs();
        let rt = inst.rt();

        let s = self.reg(rs) as i32;
        let t = self.reg(rt) as i32;
        let flag = (s < t) as u32;


        //self.handle_pending_load();
        self.set_reg(rd, flag);
    }

    /// Set on less than, unsigned
    // stlu rd,rs,rt
    // if rs < rt (unsigned comparison) then rd=1 else rd=0
    fn op_sltu(&mut self, inst: Instruction) {
        tracing::trace!("exec SLTU");
        let rd = inst.rd();
        let rs = inst.rs();
        let rt = inst.rt();

        let s = self.reg(rs);
        let t = self.reg(rt);
        let flag = (s < t) as u32;


        //self.handle_pending_load();
        self.set_reg(rd, flag);
    }

    /// Set on less than immediate
    // slti rt,rs,imm
    // if rs < imm (sign-extended immediate) (signed comparison) then rt=1, else rt=0
    fn op_slti(&mut self, inst: Instruction) {
        tracing::trace!("exec SLTI");
        let i = inst.imm_se() as i32;
        let rs = inst.rs();
        let rt = inst.rt();

        let s = self.reg(rs) as i32;
        let flag = (s < i) as u32;

        //self.handle_pending_load();
        self.set_reg(rt, flag);
    }

    /// Set on less than immediate, unsigned
    // sltiu rt,rs,imm
    // if rs < imm (sign-extended immediate) (unsigned comparison) then rt=1, else rt=0
    fn op_sltiu(&mut self, inst: Instruction) {
        tracing::trace!("exec SLTIU");
        let i = inst.imm_se();
        let rs = inst.rs();
        let rt = inst.rt();

        let s = self.reg(rs);
        let flag = (s < i) as u32;

        //self.handle_pending_load();
        self.set_reg(rt, flag);
    }


    /// Bitwise OR
    // or rd,rs,rt
    // rd = rs | rt
    fn op_or(&mut self, inst: Instruction) {
        tracing::trace!("exec OR");
        let rd = inst.rd();
        let rs = inst.rs();
        let rt = inst.rt();

        let s = self.reg(rs);
        let t = self.reg(rt);

        let val = s | t;

        //self.handle_pending_load();
        self.set_reg(rd, val);
    }

    /// Bitwise NOR
    // nor rd,rs,rt
    // rd = 0xffff_ffff ^ (rs | rt)
    fn op_nor(&mut self, inst: Instruction) {
        tracing::trace!("exec NOR");
        let rd = inst.rd();
        let rs = inst.rs();
        let rt = inst.rt();

        let s = self.reg(rs);
        let t = self.reg(rt);

        let val = 0xffff_ffff ^ (s | t);

        self.set_reg(rd, val);
    }

    /// Bitwise OR (immediate)
    // ori rs,rt,imm
    // rt = rs | (0x0000..0xffff)
    fn op_ori(&mut self, inst: Instruction) {
        tracing::trace!("exec ORI");
        let i = inst.imm();
        let rt = inst.rt();
        let rs = inst.rs();

        let s = self.reg(rs);
        let val = s | i;

        //self.handle_pending_load();
        self.set_reg(rt, val);
    }

    /// Bitwise AND
    // and rd,rs,rt
    // rd = rs & rt
    fn op_and(&mut self, inst: Instruction) {
        tracing::trace!("exec AND");

        let rd = inst.rd();
        let rs = inst.rs();
        let rt = inst.rt();

        let s = self.reg(rs);
        let t = self.reg(rt);
        let d = s & t;

        //self.handle_pending_load();
        self.set_reg(rd, d);
    }

    /// Bitwise AND immediate
    // andi rt,rs,imm
    // rt = rs & imm
    fn op_andi(&mut self, inst: Instruction) {
        tracing::trace!("exec ANDI");
        
        let i = inst.imm();
        let rt = inst.rt();
        let rs = inst.rs();

        let s = self.reg(rs); 
        let t = s & i;

        //self.handle_pending_load();
        self.set_reg(rt, t);
    }

    fn op_syscall(&mut self, _inst: Instruction) {
        tracing::trace!("exec SYSCALL");

        self.exception(Exception::Syscall);
    }

    /* === Coprocessor logic === */

    /// Invoke coprocessor 0
    // cop0 cop_op
    // exec cop0 command 0x0..0x1ff_ffff
    fn op_cop0(&mut self, bus: &mut Bus, inst: Instruction) {
        tracing::trace!("exec COP0");
        match inst.cop_op() {
            0x00 => self.op_mfc0(bus, inst),
            0x04 => self.op_mtc0(bus, inst),
            0x10 => self.op_rfe(bus, inst),
            _else => panic!("unknown cop0 delegation: {_else:05x} (op: 0x{inst:08x})"),
        }
    }

    /// Move from coprocessor 0
    // mfc0 rt,rd
    // rt = cop#_(data_reg)
    fn op_mfc0(&mut self, bus: &mut Bus, inst: Instruction) {
        tracing::trace!("delegate MFC0");
        let cpu_rt = inst.rt();
        let cop_r = inst.rd();

        let val = self.cop.mfc0(bus, cop_r);

        let load = LoadDelay::new(cpu_rt, val);
        self.chain_pending_load(load)
    }

    /// Move to coprocessor 0
    // mtc0 rt,rd
    // cop#_(data_reg) = rt
    fn op_mtc0(&mut self, bus: &mut Bus, inst: Instruction) { 
        tracing::trace!("delegate MTC0");
        let cpu_rt = inst.rt();
        let cop_r = inst.rd();

        let val = self.reg(cpu_rt);

        //self.handle_pending_load();
        self.cop.mtc0(bus, cop_r, val);
    }

    fn op_rfe(&mut self, bus: &mut Bus, inst: Instruction) {
        tracing::trace!("delegate RFE");
        if inst.inner() & 0x3f != 0b01_0000 {
            panic!("Unsupported cop0 instruction: {inst:08x}");
        }
        self.cop.pop_mode();
    }
}