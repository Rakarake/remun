#![feature(let_chains)]
pub mod addressing_modes;
pub mod memory;
pub mod opcodes;

use std::error::Error;
use std::ops::RangeInclusive;
use std::path::Path;
use std::path::PathBuf;
use std::usize;

use asmnes::assemble;
use log::debug;
use shared::AddressingMode;
use shared::BANK_SIZE;
use shared::CODEPOINTS;
use shared::Codepoint;
use shared::Opcode;
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
    /// - N  Negative
    /// - V  Overflow
    /// - \-  ignored
    /// - B  Break
    /// - D  Decimal (unused on the NES)
    /// - I  Interrupt (IRQ disable)
    /// - Z  Zero
    /// - C  Carry
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
    range: RangeInclusive<u16>,
}
pub enum Device {
    Ram(Vec<u8>),
    /// Bank index
    Rom(usize),
    Palette([u8; 32]),
}
/// There are separate address spaces, the CPU + some PPU ones
/// https://www.nesdev.org/wiki/PPU
#[derive(PartialEq)]
pub enum AddressSpace {
    Cpu,
    Ppu,
}

#[derive(Debug)]
pub enum FileError {
    AsmnesError(asmnes::AsmnesError),
    InesError(shared::InesError),
    InvalidFileType,
}

use std::fmt;
impl std::error::Error for FileError {}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::AsmnesError(e) => write!(f, "{e}"),
            FileError::InesError(e) => write!(f, "{e}"),
            FileError::InvalidFileType => write!(f, "supports files of type .nes or .asm"),
        }
    }
}

pub fn load_from_file<T: AsRef<Path>>(path: T) -> Result<Ines, FileError> {
    if let Some(os_str) = path.as_ref().extension() {
        match os_str.to_str() {
            Some("nes") => { shared::Ines::from_file(&path).map_err(FileError::InesError) }
            Some("asm") => { assemble(&path).map_err(FileError::AsmnesError) }
            _ => Err(FileError::InvalidFileType),
        }
    } else {
        Err(FileError::InvalidFileType)
    }
}

impl State {
    pub fn new(ines: Ines) -> Self {
        let pc = 0;
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
                    address_space: AddressSpace::Cpu,
                    range: 0x0000..=0x07FF,
                }],
                device: Device::Ram(vec![0; 0x0800]),
            },
        );
        if ines.inesprg == 1 {
            memory.push(
                MemoryMap {
                    memory_regions: vec![
                        MemoryRegion {
                            address_space: AddressSpace::Cpu,
                            range: 0x8000..=0x9FFF,
                        },
                        // Mirrored region
                        MemoryRegion {
                            address_space: AddressSpace::Cpu,
                            range: 0xC000..=0xDFFF,
                        },
                    ],
                    device: Device::Rom(0),
                }
            );
            memory.push(
                MemoryMap {
                    memory_regions: vec![
                        MemoryRegion {
                            address_space: AddressSpace::Cpu,
                            range: 0xA000..=0xBFFF,
                        },
                        // Mirrored region
                        MemoryRegion {
                            address_space: AddressSpace::Cpu,
                            range: 0xE000..=0xFFFF,
                        },
                    ],
                    device: Device::Rom(1),
                }
            );
        } else {
            // The other 16KiB
            memory.push(
                MemoryMap {
                    memory_regions: vec![
                        MemoryRegion {
                            address_space: AddressSpace::Cpu,
                            range: 0xC000..=0xDFFF,
                        },
                    ],
                    device: Device::Rom(3),
                }
            );
            memory.push(
                MemoryMap {
                    memory_regions: vec![
                        MemoryRegion {
                            address_space: AddressSpace::Cpu,
                            range: 0xE000..=0xFFFF,
                        },
                    ],
                    device: Device::Rom(4),
                }
            );
        }
        // 8KiB pattern memory
        memory.push(MemoryMap {
            memory_regions: vec![MemoryRegion {
                address_space: AddressSpace::Ppu,
                // TODO when working with the ppu
                range: 0x0000..=0x1FFF,
            }],
            device: Device::Rom(if ines.inesprg == 1 {3} else {5}),
        });
        // VRAM; Nametables
        let nametables_range: RangeInclusive<u16> = 0x2000..=0x3EFF;
        memory.push(MemoryMap {
            device: Device::Ram(vec![0; (nametables_range.end()+1 - nametables_range.start()) as usize]),
            memory_regions: vec![MemoryRegion {
                address_space: AddressSpace::Ppu,
                range: nametables_range,
            }],
        });
        // Palettes
        let palettes_range: RangeInclusive<u16> = 0x3F00..=0x3FFF;
        memory.push(MemoryMap {
            device: Device::Palette([0; 32]),
            memory_regions: vec![MemoryRegion {
                address_space: AddressSpace::Ppu,
                range: palettes_range,
            }],
        });

        let mut state = Self {
            pc,
            a,
            x,
            y,
            sr,
            sp,
            cycles,
            ines,
            memory,
        };
        state.reset();
        debug!("setting PC to ${:04X}", state.pc);
        state
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

    pub fn read(&mut self, address: u16, read_only: bool) -> u8 {
        self.read_from_bus(address, read_only, AddressSpace::Cpu)
    }
    pub fn write(&mut self, address: u16, value: u8) {
        self.write_to_bus(address, value, AddressSpace::Cpu);
    }
    pub fn ppu_read(&mut self, address: u16, read_only: bool) -> u8 {
        self.read_from_bus(address, read_only, AddressSpace::Ppu)
    }
    pub fn ppu_write(&mut self, address: u16, value: u8) {
        self.write_to_bus(address, value, AddressSpace::Ppu);
    }

    fn write_to_bus(&mut self, address: u16, value: u8, bus: AddressSpace) {
        debug!("write: {:#06X}", address);
        if let Some((d, r)) = try_address(&mut self.memory, bus, address) {
            match d {
                Device::Ram(bytes) => {
                    bytes[address as usize - *r.start() as usize] = value;
                }
                Device::Rom(_bytes) => {}
                Device::Palette(bs) => {
                    let address = address & 0x001F;
                    // a sprite's "transparent color"
                    let address = match address {
                        0x0010 => 0x0000,
                        0x0014 => 0x0004,
                        0x0016 => 0x0006,
                        0x0018 => 0x0008,
                        0x001C => 0x000C,
                        _ => address,
                    };
                    bs[address as usize] = value;
                }
            }
        }
    }

    /// If "read_only" is set, the read has no affect on the state of the system.
    fn read_from_bus(&mut self, address: u16, read_only: bool, bus: AddressSpace) -> u8 {
        if !read_only {
            debug!("read: {:#06X}", address);
        }
        if let Some((d, r)) = try_address(&mut self.memory, bus, address) {
            match d {
                Device::Ram(bytes) => {
                    bytes[address as usize - *r.start() as usize]
                }
                Device::Rom(i) => {
                    let index = address as usize - *r.start() as usize;
                    // This means supplied ROM does not have to be filled
                    if index < BANK_SIZE {
                        self.ines.banks[BANK_SIZE * *i + index]
                    } else {
                        0
                    }
                }
                Device::Palette(bs) => {
                    let address = address & 0x001F;
                    // a sprite's "transparent color"
                    let address = match address {
                        0x0010 => 0x0000,
                        0x0014 => 0x0004,
                        0x0016 => 0x0006,
                        0x0018 => 0x0008,
                        0x001C => 0x000C,
                        _ => address,
                    };
                    bs[address as usize]
                },
            }
        } else {
            0
        }
    }

    pub fn read_u16(&mut self, val: u16) -> u16 {
        let lo = self.read(val, false) as u16;
        let hi = self.read(val + 1, false) as u16;
        (hi << 8) | lo
    }

    /// A soft reset.
    pub fn reset(&mut self) {
        let new_pc: u16 = self.read_u16(shared::vectors::RESET);
        self.pc = new_pc;
    }

    /// Helper.
    pub fn inc_pc(&mut self) {
        let (new_pc, _) = self.pc.overflowing_add(1);
        self.pc = new_pc;
    }
}

/// Helper function to get the device and the range
fn try_address(
    memory: &mut Vec<MemoryMap>,
    address_space: AddressSpace,
    address: u16,
) -> Option<(&mut Device, RangeInclusive<u16>)> {
    memory.iter_mut().find_map(|m| {
        m.memory_regions
            .iter()
            .find(|mr| mr.address_space == address_space && mr.range.contains(&address))
            .map(|mr| (&mut m.device, mr.range.clone()))
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
