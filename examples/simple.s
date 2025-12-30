; Simple test for Sampo assembler
        .org 0x0100

start:
        ADD  R4, R5, R6         ; 0x0456
        SUB  R4, R5, R6         ; 0x1456
        ADDI R4, 10             ; 0x540A
        LW   R4, (R5)           ; 0x6450
        SW   (R5), R4           ; 0x7450
        BEQ  next               ; branch forward
        J    start              ; jump back
next:
        PUSH R4                 ; push
        POP  R5                 ; pop
        MOV  R4, R5             ; move
        NOP                     ; no-op
        HALT                    ; halt
