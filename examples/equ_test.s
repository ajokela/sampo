; Test .equ directive
        .org 0x0100

.equ    PORT_A  0x80
.equ    PORT_B  0x81

start:
        INI  R4, PORT_A         ; Should read from port 0x80
        HALT
