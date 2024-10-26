SECTIONS {
    .text : {
        *(.text .text.*)

        KEEP(*(.module))
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : {
        *(.data .data.*)

        KEEP(*(.info))
    }

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    }
}