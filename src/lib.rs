//! The memory bus of the system, owns all other components.

mod bus;
mod cpu;

const OMEGAWOW: i32 = bus::WOW;

const RAM_RANGE: (u16, u16) = (0x0000, 0xFFFF);

mod s_flags {
    const C: u8 = 1 << 0;  // Carry
    const Z: u8 = 1 << 1;  // Zero
    const I: u8 = 1 << 2;  // Disable Interrupts
    const D: u8 = 1 << 3;  // Decimal Mode (Unused in NES)
    const B: u8 = 1 << 4;  // Break
    const U: u8 = 1 << 5;  // Unused :(
    const V: u8 = 1 << 6;  // Overflow
    const N: u8 = 1 << 7;  // Negative
}

// Opcodes indexed by their binary first byte
// (0,0) is top-left
// MSD: Most Segnificant Digit, indexes down
// LSD: Least Segnificant Digit, indexes right
const OPCODES: [[Opcode; 16]; 16] = [
    [ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL],
    [ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL],
    [ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL],
    [ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL],
    [ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL],
    [ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL],
    [ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL],
    [ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL],
    [ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL],
    [ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL],
    [ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL],
    [ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL],
    [ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL],
    [ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL],
    [ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL],
    [ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL,ILL],
];

struct Opcode<'a> {
    // Name is the mnemonic code
    name: &'a str,
    op_func: fn(&mut CPU) -> u8,
    addr_func: fn(&mut CPU) -> u8,
    // Number of cycles required to execute
    cycles: u8,
}

// All opcodes
const ILL: Opcode = Opcode { name: "ILL", op_func: ill, addr_func: imm, cycles: 1 };

// Information from the addressing mode operation to the opcode operation,
// this state does not "exist" in hardware, it can be seen as
// generic data used in opcodes so we don't get a lot of redundant code.
// There is redundant data here, since all instructions does not use
// all the fields.
struct instr_data {
    // All the absolute, zero-page absolute instructions, indirect pointer instructions
    addr_abs: u16,
    addr_rel: u16,
}

// All addressing mode functions, may return extra clock cycles
// Implied (no data)
fn imp(cpu: &mut CPU) -> u8 {
    cpu.fetched = cpu.a;
    0
}
fn imm(cpu: &mut CPU) -> u8 {
    0
}

fn abs(cpu: &mut CPU) -> u8 {
    0
}
// NOTE: indicate an extra cycle on page overflow
fn abx(cpu: &mut CPU) -> u8 {
    0
}

// NOTE: Hardware bug page edge-case
fn ind(cpu: &mut CPU) -> u8 {
    0
}

// All opcode functions, may return extra clock cycles
fn ill(cpu: &mut CPU) -> u8 {
    println!("Illegale Opcode: OH NOOOOO!!!");
    0
}

pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    stack_pointer: u8,
    pc: u16,
    status: u8,

    bus: Bus,
    // The memory to be worked on
    fetched: u8,
}

impl CPU {
    // Read/Write to memory requires a "bus"
    fn read_from_bus(&mut self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    fn write_to_bus(&mut self, addr: u16, data: u8) {
        self.bus.write(addr, data);
    }

    // Exectue the next instruction
    fn execute_next(&mut self) {
        // Read the next opcode
        let opcode = self.read_from_bus(self.pc);
        // Index the table
        // TODO: check which part of the opcode byte is h/v index
        let v_index = opcode & 0xF0;  // Most significant
        let h_index = opcode & 0x0F;  // Least significant
        let abstract_opcode: &Opcode = &OPCODES[v_index as usize][h_index as usize];
        let extra_cycles = (abstract_opcode.op_func)(self);
        self.pc += (abstract_opcode.cycles + extra_cycles) as u16;
    }

    // Interrupts
    fn reset(&mut self) {}
    // Normal interrupt: can be turned off
    fn irq(&mut self) {}
    // Non-Maskable-Interrupt, cannot be turned off
    fn nmi(&mut self) {}
}

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

