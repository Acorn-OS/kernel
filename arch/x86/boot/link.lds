SECTIONS{
    .boot.entry : { *(.boot.entry)  *(.boot.entry.*) }
    . = ALIGN(512);
    .boot.text  : { *(.boot.text)   *(.boot.text.*)  }
}
