.inesmap 0
.inesmir 1
.inesprg 1
.ineschr 1

.bank 0
.bank 1
.bank 2

.bank 0
.org $A000

; All addressing modes!
NOP           ; Implied
LDA #12       ; Immediate
BNE $12       ; Relative
ROR A         ; A
LDA $12       ; Zero Page
LDA $12,X     ; Zero Page X indexed
LDA $12,Y     ; Zero Page X indexed
LDA $1234     ; Absolute
LDA $1234,X   ; Aboslute X indexed
LDA $1234,Y   ; Aboslute Y indexed
LDA ($1234)   ; Indirect
LDA ($12,X)   ; Indirect X indexed
LDA ($12),Y   ; Indirect Y indexed

; Opcodes 🕹️
LDA #12
CLC
ADC 3

