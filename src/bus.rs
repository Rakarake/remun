//! The memory bus of the system
const RAM_RANGE: (u16, u16) = (0x0000, 0xFFFF);
pub struct Bus {
    //TODO: make not take up whole address range
    ram: [u8; 64 * 1024],
}

fn inside_address_range(addr: u16, range: (u16, u16)) -> bool {
    addr >= range.0 && addr <= range.1
}

impl Bus {
    fn read(&mut self, addr: u16) -> u8 {
        if inside_address_range(addr, RAM_RANGE) {
            return self.ram[addr as usize]
        }
        0x0000
    }

    fn write(&mut self, addr: u16, data: u8) {
        
    }
}
