PHDRS
{
    text    PT_LOAD FLAGS(0x5);
    rodata  PT_LOAD FLAGS(0x4);
}

SECTIONS
{
    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .text : {
        *(.text .text.*)
    } :text

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    /DISCARD/ : {
        *(.data .data.*)
        *(.eh_frame*)
        *(.bss .bss.*)
        *(COMMON)
        *(.note .note.*)
    }
}
