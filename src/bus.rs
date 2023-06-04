type Nrom128Prg = [u8 ; 1024 * 16];
type Nrom128Chr = [u8 ; 1024 * 8];

pub struct Bus {
    // Memory map
    ranges: Vec<Box<dyn BusRange>>,
    prg: Nrom128Prg,
    chr: Nrom128Chr,
}

impl Bus {
    // Only NROM mapper for now.
    fn new_nrom_128(prg: Nrom128Prg, chr: Nrom128Chr) {
        
    }

    fn read(&mut self, addr: u16) -> u8 {
        match self.ranges.iter_mut().find(|range| is_in_range(range.range(), addr)) {
            None => 0, // TODO: emulate non-mapped read regions
            Some(range) => range.read(addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match self.ranges.iter_mut().find(|range| is_in_range(range.range(), addr)) {
            None => (), // TODO: emulate non-mapped write regions
            Some(range) => range.write(addr, data),
        }
    }
}

// Inclusive range check
fn is_in_range(range: (u16, u16), val: u16) -> bool {
    val >= range.0 && val <= range.1
}

// Trait of a device connected to a memory region.
// Regions for reading and writing are not separate.
pub trait BusRange {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
    // Inclusive range
    fn range(&self) -> (u16, u16);
}

