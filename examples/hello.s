; Hello World for Sampo CPU
; Outputs "Hello, Sampo!" to serial port

        .org 0x0100

; Serial port definitions (MC6850 ACIA style)
.equ    ACIA_STATUS 0x80
.equ    ACIA_DATA   0x81
.equ    TX_READY    0x02

start:
        LIX  R4, message        ; Load address of message

loop:
        LBU  R5, (R4)           ; Load byte from string
        CMP  R5, R0             ; Compare with zero
        BEQ  done               ; If null, we're done

wait_tx:
        INI  R6, ACIA_STATUS    ; Read serial status
        AND  R7, R6, R6         ; Copy to R7
        ADDI R7, -2             ; Subtract TX_READY (2)
        BNE  wait_tx            ; Wait if not ready

        OUTI ACIA_DATA, R5      ; Write character
        ADDI R4, 1              ; Next character
        J    loop               ; Continue

done:
        HALT

message:
        .asciz "Hello, Sampo!\n"
