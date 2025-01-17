macro_rules! another_nord_w {
    () => {
        5
    };
}


fn main() {
    println!("Hello, world!");
}

struct State {
    pc: u16,
    a: u8,
}

// Instruction lookup

// Opcode
enum Opcode {
    JAM,
    ADC,
}

impl Opcode {
    fn run(&self, state: &mut State) {
    }
}

// Addressing modes
enum AddressingMode {
    IMPL,  // nuh uh
    REL,   // $FF
    ABS,   // $FFFF
    ABS_X, // $FFFF,X
    ABS_Y, // $FFFF,Y
    IMM,   // #FF
    IND,   // ($FFFF)
    X_IND, // ($FF,X)
    IND_Y, // ($FF),Y
    ZPG,   // $FF
    ZPG_X, // $FF,X
    ZPG_Y, // $FF,Y
}

impl AddressingMode {
    fn run(&self, state: &mut State) {
    }
}

// Instructions
struct Instruction {
    opcode: Opcode,
    addressing_mode: AddressingMode,
    //cycles: u32,
}

//const INSTRUCTIONS: [Instruction; 256] = [
//];

const W: u8 = another_nord_w!();

macro_rules! tabalize {
    () => {
        
    };
}

mod instruction_stuff {
    use crate::AddressingMode::*;
    use crate::AddressingMode;
    use crate::Instruction::*;
    use crate::Instruction;

    const INSTRUCTIONS: [Instruction; 256] = [
    BRK impl ; ORA X,ind ; JAM   ; SLO X,ind ; NOP zpg   ; ORA zpg   ; ASL zpg   ; SLO zpg   ; PHP impl ; ORA #     ;  ASL A    ; ANC #     ; NOP abs   ; ORA abs   ; ASL abs   ; SLO abs   ;
    BPL rel  ; ORA ind_Y ; JAM   ; SLO ind_Y ; NOP zpg_X ; ORA zpg_X ; ASL zpg_X ; SLO zpg_X ; CLC impl ; ORA abs_Y ;  NOP impl ; SLO abs_Y ; NOP abs_X ; ORA abs_X ; ASL abs_X ; SLO abs_X ;
    JSR abs  ; AND X,ind ; JAM   ; RLA X,ind ; BIT zpg   ; AND zpg   ; ROL zpg   ; RLA zpg   ; PLP impl ; AND #     ;  ROL A    ; ANC #     ; BIT abs   ; AND abs   ; ROL abs   ; RLA abs   ;
    BMI rel  ; AND ind_Y ; JAM   ; RLA ind_Y ; NOP zpg_X ; AND zpg_X ; ROL zpg_X ; RLA zpg_X ; SEC impl ; AND abs_Y ;  NOP impl ; RLA abs_Y ; NOP abs_X ; AND abs_X ; ROL abs_X ; RLA abs_X ;
    RTI impl ; EOR X,ind ; JAM   ; SRE X,ind ; NOP zpg   ; EOR zpg   ; LSR zpg   ; SRE zpg   ; PHA impl ; EOR #     ;  LSR A    ; ALR #     ; JMP abs   ; EOR abs   ; LSR abs   ; SRE abs   ;
    BVC rel  ; EOR ind_Y ; JAM   ; SRE ind_Y ; NOP zpg_X ; EOR zpg_X ; LSR zpg_X ; SRE zpg_X ; CLI impl ; EOR abs_Y ;  NOP impl ; SRE abs_Y ; NOP abs_X ; EOR abs_X ; LSR abs_X ; SRE abs_X ;
    RTS impl ; ADC X,ind ; JAM   ; RRA X,ind ; NOP zpg   ; ADC zpg   ; ROR zpg   ; RRA zpg   ; PLA impl ; ADC #     ;  ROR A    ; ARR #     ; JMP ind   ; ADC abs   ; ROR abs   ; RRA abs   ;
    BVS rel  ; ADC ind_Y ; JAM   ; RRA ind_Y ; NOP zpg_X ; ADC zpg_X ; ROR zpg_X ; RRA zpg_X ; SEI impl ; ADC abs_Y ;  NOP impl ; RRA abs_Y ; NOP abs_X ; ADC abs_X ; ROR abs_X ; RRA abs_X ;
    NOP #    ; STA X,ind ; NOP # ; SAX X,ind ; STY zpg   ; STA zpg   ; STX zpg   ; SAX zpg   ; DEY impl ; NOP #     ;  TXA impl ; ANE #     ; STY abs   ; STA abs   ; STX abs   ; SAX abs   ;
    BCC rel  ; STA ind_Y ; JAM   ; SHA ind_Y ; STY zpg_X ; STA zpg_X ; STX zpg_Y ; SAX zpg_Y ; TYA impl ; STA abs_Y ;  TXS impl ; TAS abs_Y ; SHY abs_X ; STA abs_X ; SHX abs_Y ; SHA abs_Y ;
    LDY #    ; LDA X,ind ; LDX # ; LAX X,ind ; LDY zpg   ; LDA zpg   ; LDX zpg   ; LAX zpg   ; TAY impl ; LDA #     ;  TAX impl ; LXA #     ; LDY abs   ; LDA abs   ; LDX abs   ; LAX abs   ;
    BCS rel  ; LDA ind_Y ; JAM   ; LAX ind_Y ; LDY zpg_X ; LDA zpg_X ; LDX zpg_Y ; LAX zpg_Y ; CLV impl ; LDA abs_Y ;  TSX impl ; LAS abs_Y ; LDY abs_X ; LDA abs_X ; LDX abs_Y ; LAX abs_Y ;
    CPY #    ; CMP X,ind ; NOP # ; DCP X,ind ; CPY zpg   ; CMP zpg   ; DEC zpg   ; DCP zpg   ; INY impl ; CMP #     ;  DEX impl ; SBX #     ; CPY abs   ; CMP abs   ; DEC abs   ; DCP abs   ;
    BNE rel  ; CMP ind_Y ; JAM   ; DCP ind_Y ; NOP zpg_X ; CMP zpg_X ; DEC zpg_X ; DCP zpg_X ; CLD impl ; CMP abs_Y ;  NOP impl ; DCP abs_Y ; NOP abs_X ; CMP abs_X ; DEC abs_X ; DCP abs_X ;
    CPX #    ; SBC X,ind ; NOP # ; ISC X,ind ; CPX zpg   ; SBC zpg   ; INC zpg   ; ISC zpg   ; INX impl ; SBC #     ;  NOP impl ; USBC #    ; CPX abs   ; SBC abs   ; INC abs   ; ISC abs   ;
    BEQ rel  ; SBC ind_Y ; JAM   ; ISC ind_Y ; NOP zpg_X ; SBC zpg_X ; INC zpg_X ; ISC zpg_X ; SED impl ; SBC abs_Y ;  NOP impl ; ISC abs_Y ; NOP abs_X ; SBC abs_X ; INC abs_X ; ISC abs_X ;
    ];
}

