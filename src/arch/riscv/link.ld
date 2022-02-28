/* OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH("i386:x86-64") */
ENTRY(_start)
phys = 0x0000000080200000;

SECTIONS
{
  kernel_start = phys;
  .text phys : AT(ADDR(.text)) {
    *(.text._start)
    *(.text.*)
  }
  .rodata ALIGN(4096) : AT(ADDR(.rodata)) {
    *(.rodata)
    *(.rodata.*)
  }
  .data ALIGN(4096) : AT(ADDR(.data)) {
    *(.data)
    *(.data.*)
  }
  .bss ALIGN(4096) : AT(ADDR(.bss)) {
    bss_start = .;
    *(.bss)
    *(.bss.*)
  }
  .sbss ALIGN(4096) : AT(ADDR(.sbss)) {
    *(.sbss)
    *(.sbss.*)
  }
  bss_end = .;
  . = ALIGN(4096);
  __boot_core_stack_start = .;         /*   ^             */
                                       /*   | stack       */
  . += 512K;                           /*   | growth      */
                                       /*   | direction   */
  __boot_core_stack_end_exclusive = .; /*   |             */
  kernel_end = .;
}
