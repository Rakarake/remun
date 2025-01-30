use crate::Range;
use crate::Device;

const RAM_RANGE: Range = Range(0x0000, 0x0100);

pub struct RAM<const N: usize> {
    pub range: Range,
    pub memory: [u8; N],  
}

impl<const N: usize> Device for RAM<N> {
    fn get_range(&self) -> Range {
        self.range
    }
    fn read(&mut self, address: u16) -> u8 {
        self.memory[address as usize]
    }
    fn write(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }
}

pub struct ROM<const N: usize> {
    pub range: Range,
    pub memory: [u8; N],  
}

impl<const N: usize> Device for ROM<N> {
    fn get_range(&self) -> Range {
        self.range
    }
    fn read(&mut self, address: u16) -> u8 {
        self.memory[address as usize]
    }
    fn write(&mut self, _address: u16, _value: u8) { }
}

