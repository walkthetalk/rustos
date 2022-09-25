
OUTPUT_ARCH(riscv)
ENTRY(_start)

ASSERT(BASE_ADDRESS, "base_adress must be not zero!")

SECTIONS
{
    . = BASE_ADDRESS;
    .text : {
        QUAD(BASE_ADDRESS)
        *(.text.entry)
        *(.text .text.*)
    }
    . = ALIGN(4K);
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    . = ALIGN(4K);
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    .bss : {
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
    ebss = .;
    /DISCARD/ : {
        *(.eh_frame)
        *(.debug*)
    }
}
