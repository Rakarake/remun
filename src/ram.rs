use crate::bus::BusRange;

// Simple module describing the built in RAM.
pub struct Ram {
    ram: [u8; 0x800],
}

impl BusRange for Ram {
    fn read(&mut self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }
    fn range(&self) -> (u16, u16) {
        (0x0000, 0x07FF)
    }
}
