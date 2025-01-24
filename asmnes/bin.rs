use shared::INSTRUCTIONS;
use asmnes::MORG;
use asmnes::simple_assemble;

fn main() {
    println!("hello!");
    println!("{:?}", simple_assemble("hello").unwrap());
}

