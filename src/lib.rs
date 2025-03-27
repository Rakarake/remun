#![feature(let_chains)]
pub mod addressing_modes;
pub mod memory;
pub mod opcodes;

use std::usize;

use log::debug;
use shared::AddressingMode;
use shared::BANK_SIZE;
use shared::CODEPOINTS;
use shared::Codepoint;
use shared::Opcode;
use shared::Range;
use shared::Ines;

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
    /// Number of cycles that have passed.
    pub cycles: u64,
    /// The static cartridge information.
    pub ines: Ines,
    /// The dynamic memory mappings.
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
    /// Bank index
    ROM(usize),
}
/// There are separate address spaces, the CPU + some PPU ones
/// https://www.nesdev.org/wiki/PPU
#[derive(PartialEq)]
pub enum AddressSpace {
    CPU,
    PPU,
}

impl State {
    pub fn new(ines: Ines) -> Self {
        let pc = 0xc000;
        let x = 0;
        let a = 0;
        let y = 0;
        let sr = 0;
        let sp = 0xFF;
        let cycles = 0;
        let mut memory: Vec<MemoryMap> = Vec::new();
        memory.push(
            MemoryMap {
                memory_regions: vec![MemoryRegion {
                    address_space: AddressSpace::CPU,
                    range: Range(0x0000, 0x07FF),
                }],
                device: Device::RAM(vec![0; 0x0800]),
            },
        );
        if ines.inesprg == 1 {
            memory.push(
                MemoryMap {
                    memory_regions: vec![
                        MemoryRegion {
                            address_space: AddressSpace::CPU,
                            range: Range(0x8000, 0x9FFF),
                        },
                        // Mirrored region
                        MemoryRegion {
                            address_space: AddressSpace::CPU,
                            range: Range(0xC000, 0xDFFF),
                        },
                    ],
                    device: Device::ROM(0),
                }
            );
            memory.push(
                MemoryMap {
                    memory_regions: vec![
                        MemoryRegion {
                            address_space: AddressSpace::CPU,
                            range: Range(0xA000, 0xBFFF),
                        },
                        // Mirrored region
                        MemoryRegion {
                            address_space: AddressSpace::CPU,
                            range: Range(0xE000, 0xFFFF),
                        },
                    ],
                    device: Device::ROM(1),
                }
            );
        } else {
            // The other 16KiB
            memory.push(
                MemoryMap {
                    memory_regions: vec![
                        MemoryRegion {
                            address_space: AddressSpace::CPU,
                            range: Range(0xC000, 0xDFFF),
                        },
                    ],
                    device: Device::ROM(3),
                }
            );
            memory.push(
                MemoryMap {
                    memory_regions: vec![
                        MemoryRegion {
                            address_space: AddressSpace::CPU,
                            range: Range(0xE000, 0xFFFF),
                        },
                    ],
                    device: Device::ROM(4),
                }
            );
        }
        // chr
        memory.push(MemoryMap {
            memory_regions: vec![MemoryRegion {
                address_space: AddressSpace::PPU,
                // TODO when working with the ppu
                range: Range(0x0000, 0x0000),
            }],
            device: Device::ROM(if ines.inesprg == 1 {3} else {5}),
        });
        Self {
            pc,
            a,
            x,
            y,
            sr,
            sp,
            cycles,
            ines,
            memory,
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
        let instr = self.read(self.pc, false);
        let Codepoint {
            opcode,
            addressing_mode,
        } = CODEPOINTS[instr as usize].clone();
        debug!("running {:?} {:?} at ${:04X}", opcode, addressing_mode, self.pc);
        let memory_target = addressing_modes::run(addressing_mode, self);
        opcodes::run(opcode, self, memory_target);
    }

    pub fn run_instructions(&mut self, n_instructions: u64) {
        for _ in 0..n_instructions {
            self.run_one_instruction();
        }
    }

    pub fn print_state(&self) {
        println!(
            "\
pc: {:#06X}
a: {:#06X}
x: {:#06X}
y: {:#06X}
sr: {:#06X}
sp: {:#06X}
cycles: {}\
",
            self.pc, self.a, self.x, self.y, self.sr, self.sp, self.cycles
        );
    }

    /// If "read_only" is set, the read has no affect on the state of the system.
    pub fn read(&mut self, address: u16, read_only: bool) -> u8 {
        if !read_only {
            debug!("read: {:#06X}", address);
        }
        if let Some((d, r)) = try_address(&mut self.memory, AddressSpace::CPU, address) {
            match d {
                Device::RAM(bytes) => {
                    return bytes[address as usize - r.0 as usize];
                }
                Device::ROM(i) => {
                    let index = address as usize - r.0 as usize;
                    // This means supplied ROM does not have to be filled
                    if index < BANK_SIZE {
                        return self.ines.banks[BANK_SIZE * *i + index]
                    } else {
                        return 0;
                    }
                }
            }
        }
        0
    }
    pub fn write(&mut self, address: u16, value: u8) {
        debug!("write: {:#06X}", address);
        if let Some((d, r)) = try_address(&mut self.memory, AddressSpace::CPU, address) {
            match d {
                Device::RAM(bytes) => {
                    bytes[address as usize - r.0 as usize] = value;
                }
                Device::ROM(_bytes) => {}
            }
        }
    }
}

/// Helper function to get the device and the range
fn try_address(
    memory: &mut Vec<MemoryMap>,
    address_space: AddressSpace,
    address: u16,
) -> Option<(&mut Device, Range)> {
    memory.iter_mut().find_map(|m| {
        m.memory_regions
            .iter()
            .find(|mr| mr.address_space == address_space && mr.range.contains(address))
            .map(|mr| (&mut m.device, mr.range))
    })
}

/// An addressing mode addresses memory, one of these.
/// Data output form addressing mode, input to opcode
#[derive(Debug, PartialEq, Eq)]
pub enum MemoryTarget {
    Address(u16),
    Accumulator,
    Impl,
}
