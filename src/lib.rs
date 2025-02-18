pub mod opcodes;
pub mod addressing_modes;
pub mod memory;

use std::usize;

use shared::Opcode;
use shared::AddressingMode;
use shared::Codepoint;
use shared::CODEPOINTS;
use shared::Range;

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
    /// The devices
    pub memory: Vec<MemoryMap>,
}

/// A device with can be mapped to memory regions on the cpu-bus or the ppu-bus
pub struct MemoryMap {
    memory_regions: Vec<MemoryRegion>,
    device: Device,
}
/// A memory range on the cpu or ppu
pub struct MemoryRegion {
    /// Memory could be on cpu/ppu
    address_space: AddressSpace,
    range: Range,
}
pub enum Device {
    RAM(Vec<u8>),
    ROM(Vec<u8>),
}
/// There are separate address spaces, the CPU + some PPU ones
/// https://www.nesdev.org/wiki/PPU
#[derive(PartialEq)]
pub enum AddressSpace {
    CPU,
    PPU,
}

impl State {
    /// prg: 16KiB, chr: 8KiB
    pub fn new_nrom128(prg: Vec<u8>, chr: Vec<u8>) -> Self {
        Self {
            pc: 0xC000,
            a: 0,
            x: 0,
            y: 0,
            sr: 0,
            sp: 0xFF,
            cycles: 0,
            memory: vec![
                // built in ram
                MemoryMap {
                    memory_regions: vec![
                        MemoryRegion {
                            address_space: AddressSpace::CPU,
                            range: Range(0x0000, 0x0800),
                        },
                    ],
                    device: Device::RAM(vec![0 ; 0x0800]),
                },
                // prg
                MemoryMap {
                    memory_regions: vec![
                        MemoryRegion {
                            address_space: AddressSpace::CPU,
                            range: Range(0x8000, 0xBFFF),
                        },
                        // mirrored
                        MemoryRegion {
                            address_space: AddressSpace::CPU,
                            range: Range(0xC000, 0xFFFF),
                        },
                    ],
                    device: Device::ROM(prg),
                },
                // chr
                MemoryMap {
                    memory_regions: vec![
                        MemoryRegion {
                            address_space: AddressSpace::PPU,
                            // TODO when working with the ppu
                            range: Range(0x0000, 0x0000),
                        },
                    ],
                    device: Device::ROM(chr),
                },
            ],
        }
    }
    
    pub fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.sr |= flag;
        } else {
            self.sr &= !flag;
        }
    }

    pub fn get_flag(&self, flag: u8) -> bool {
        (self.sr & flag) == flag
    }

    pub fn run_one_instruction(&mut self) {
        let instr = self.read(self.pc);
        let Codepoint { opcode, addressing_mode } = CODEPOINTS[instr as usize].clone();
        let memory_target = addressing_modes::run(addressing_mode, self);
        opcodes::run(opcode, self, memory_target);
    }

    pub fn run_instructions(&mut self, n_instructions: u64) {
        for _ in 0..n_instructions {
            self.run_one_instruction();
        }
    }

    pub fn print_state(&self) {
        println!("\
pc: {:#06X}
a: {:#06X}
x: {:#06X}
y: {:#06X}
sr: {:#06X}
sp: {:#06X}
cycles: {}\
", self.pc, self.a, self.x, self.y, self.sr, self.sp, self.cycles);
    }

    /// Helper function to get the device and the range
    fn try_address(&mut self, address_space: AddressSpace, address: u16) -> Option<(&mut Device, Range)> {
        self.memory.iter_mut().find_map(|m| {
            m.memory_regions.iter().find(|mr|
                mr.address_space == address_space &&
                mr.range.contains(address)
            ).map(|mr| (&mut m.device, mr.range))
        })
    }
    pub fn read(&mut self, address: u16) -> u8 {
        if let Some((d, r)) = self.try_address(AddressSpace::CPU, address) {
            match d {
                Device::RAM(bytes) => {
                    return bytes[address as usize - r.0 as usize];
                },
                Device::ROM(bytes) => {
                    let index = address as usize - r.0 as usize;
                    // This means supplied ROM does not have to be filled
                    if index < bytes.len() {
                        return bytes[index];
                    } else {
                        return 0;
                    }
                },
            }
        }
        return 0;
    }
    pub fn write(&mut self, address: u16, value: u8) {
        println!("hello!, {:?}", address);
        if let Some((d, r)) = self.try_address(AddressSpace::CPU, address) {
            match d {
                Device::RAM(bytes) => {
                    bytes[address as usize - r.0 as usize] = value;
                },
                Device::ROM(bytes) => { },
            }
        }
    }
}

/// An addressing mode addresses memory, one of these.
/// Data output form addressing mode, input to opcode
#[derive(Debug, PartialEq, Eq)]
pub enum MemoryTarget {
    Address(u16),
    Accumulator,
    Impl,
}

