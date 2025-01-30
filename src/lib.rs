pub mod opcodes;
pub mod addressing_modes;
pub mod memory;

use shared::Opcode;
use shared::AddressingMode;
use shared::Instruction;
use shared::INSTRUCTIONS;

/// `0`: inclusive, `1`: exclusive
#[derive(Clone, Copy)]
pub struct Range(pub u16, pub u16);
impl Range {
    fn contains(&self, value: u16) -> bool {
        value >= self.0 && value < self.1
    }
}

/// The state of the NES, registers, all devices mapped to memory-regions
pub struct State {
    /// Program counter
    pub pc: u16,            
    /// Accumulator register
    pub a: u8,              
    /// X register
    pub x: u8,              
    /// Y register
    pub y: u8,              
    /// Status register: NV-BDIZC
    /// N  Negative
    /// V  Overflow
    /// -  ignored
    /// B  Break
    /// D  Decimal (unused on the NES)
    /// I  Interrupt (IRQ disable)
    /// Z  Zero
    /// C  Carry
    pub sr: u8,             
    /// Stack pointer
    pub sp: u8,             
    /// Number of cycles that have passed
    pub cycles: u64,        
    pub devices: Vec<Box<dyn Device>>,
}

pub trait Device {
    fn get_range(&self) -> Range;
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

impl State {
    pub fn run_one_instruction(&mut self) {
        let instr = self.read(self.pc);
        let Instruction { opcode, addressing_mode } = INSTRUCTIONS[instr as usize].clone();
        let memory_target = addressing_modes::run(addressing_mode, self);
        opcodes::run(opcode, self, memory_target);
    }

    pub fn print_state(&self) {
        println!("\
pc: {}
a: {}
x: {}
y: {}
sr: {}
sp: {}
cycles: {}\
", self.pc, self.a, self.x, self.y, self.sr, self.sp, self.cycles);
    }

    pub fn read(&mut self, address: u16) -> u8 {
        // TODO check that multiple devices overlap
        for device in self.devices.iter_mut() {
            if device.get_range().contains(address) {
                return device.read(address)
            }
        }
        unimplemented!("memory range: {}", address);
    }
    pub fn write(&mut self, address: u16, value: u8) {
        // TODO check that multiple devices overlap
        for device in self.devices.iter_mut() {
            if device.get_range().contains(address) {
                device.write(address, value);
                return;
            }
        }
        unimplemented!("memory range: {}", address);
    }
}

/// An addressing mode addresses memory, one of these.
#[derive(Debug, PartialEq, Eq)]
pub enum MemoryTarget {
    Address(u16),
    Accumulator,
    Impl,
}

