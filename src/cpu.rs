use crate::ic::Interconnect;
use crate::bios::BIOS_START;

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

    pub fn step(&mut self) {
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