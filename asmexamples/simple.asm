; All addressing modes!
.inesprg 1
.bank 0

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

