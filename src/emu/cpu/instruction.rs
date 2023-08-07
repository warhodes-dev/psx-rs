//! Module for Instruction and its related types, such as LoadDelay and RegisterIndex

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RegisterIndex(pub u32);

impl RegisterIndex {
    pub const RETURN: RegisterIndex = RegisterIndex(31);
}

impl From<RegisterIndex> for u32 {
    fn from(r: RegisterIndex) -> Self {
        r.0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Instruction(pub u32);

impl Instruction {
    /// Return bits [31:26] of instruction
    pub fn opcode(self) -> u32 {
        let Instruction(op) = self;
        op >> 26
    }

    /// Return subfunction value in bits [5:0]
    pub fn funct(self) -> u32 {
        let Instruction(op) = self;
        op & 0x3f
    }

    /// Return coprocessor opcode value in bits [25:21]
    pub fn cop_op(self) -> u32 {
        let Instruction(op) = self;
        let i = (op >> 21) & 0x1f;
        i
    }

    /// Return register index in bits [25:21]
    pub fn rs(self) -> RegisterIndex {
        let Instruction(op) = self;
        let i = (op >> 21) & 0x1f;
        RegisterIndex(i)
    }

    /// Return register index in bits [20:16]
    pub fn rt(self) -> RegisterIndex {
        let Instruction(op) = self;
        let i = (op >> 16) & 0x1f;
        RegisterIndex(i)
    }

    /// Return register index in bits [15:11]
    pub fn rd(self) -> RegisterIndex {
        let Instruction(op) = self;
        let i = (op >> 11) & 0x1f;
        RegisterIndex(i)
    }

    /// Return 'Shift amount immediate' value in bits [10:6]
    pub fn shamt(self) -> u32 {
        let Instruction(op) = self;
        (op >> 6) & 0x1f
    }

    /// Return immediate value in bits [16:0]
    pub fn imm(self) -> u32 {
        let Instruction(op) = self;
        op & 0xffff
    }

    /// Return immediate value in bits [16:0] as a sign-extended 32 bit value
    pub fn imm_se(self) -> u32 {
        let Instruction(op) = self;
        let sign_extend = |n: u32| { n as i16 as u32 };
        sign_extend(op & 0xffff)
    }

    /// Return immediate value in bits [25:0] used for jump address
    pub fn addr(self) -> u32 {
        let Instruction(op) = self;
        op & 0x3ff_ffff
    }

    pub fn inner(self) -> u32 {
        let Instruction(inner) = self;
        inner
    }
}