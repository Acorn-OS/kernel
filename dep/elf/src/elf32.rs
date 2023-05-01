use core::fmt::{Debug, Display};
use core::mem::size_of;
use core::ops::Deref;
use core::ptr;

use alloc::boxed::Box;
use alloc::string::{String, ToString};

#[derive(Debug)]
pub enum Error {
    InvalidHeaderMagic,
    InsufficientSizeForHeader(usize),
    MalformedHeader(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&match self {
            Error::InvalidHeaderMagic => "invalid header magic".to_string(),
            Error::InsufficientSizeForHeader(s) => {
                format!("insufficient binary space to fit ELF header with a size of '{s}'")
            }
            Error::MalformedHeader(e) => format!("malformed elf '{e}'"),
        })
    }
}

impl core::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;

type Elf32Half = u16;
type Elf32Word = u32;
type Elf32Sword = i32;
type Elf32Addr = u32;
type Elf32Off = u32;

const EI_CLASS: usize = 4;
const EI_DATA: usize = 5;
const EI_VERSION: usize = 6;
const EI_PAD: usize = 7;
const EI_NIDENT: usize = 16;

const ET_NONE: Elf32Half = 0;
const ET_REL: Elf32Half = 1;
const ET_EXEC: Elf32Half = 2;
const ET_DYN: Elf32Half = 3;
const ET_CORE: Elf32Half = 4;
const ET_LOPROC: Elf32Half = 0xff00;
const ET_HIPROC: Elf32Half = 0xffff;

const EM_NONE: Elf32Half = 0;
const EM_M32: Elf32Half = 1;
const EM_SPARC: Elf32Half = 2;
const EM_386: Elf32Half = 3;
const EM_68K: Elf32Half = 4;
const EM_88K: Elf32Half = 5;
const EM_860: Elf32Half = 7;
const EM_MIPS: Elf32Half = 8;

const EV_NONE: Elf32Word = 0;
const EV_CURRENT: Elf32Word = 1;

const ELFCLASSNONE: u8 = 0;
const ELFCLASS32: u8 = 1;
const ELFCLASS64: u8 = 2;

const ELFDATANONE: u8 = 0;
const ELFDATA2LSB: u8 = 1;
const ELFDATA2MSB: u8 = 2;

#[derive(Debug)]
#[repr(C)]
pub struct Elf32Ehdr {
    /// The initial bytes mark the file as an object file and provide machine-independent data
    /// with which to decode and interpret the file’s contents.
    e_ident: [u8; EI_NIDENT],
    /// This member identifies the object file type.
    /// Although the core file contents are unspecified, type `ET_CORE` is reserved to mark the
    /// file. Values from `ET_LOPROC` through `ET_HIPROC` (inclusive) are reserved for
    /// processor-specific semantics. Other values are reserved and will be assigned to new
    /// object file types as necessary.
    e_type: Elf32Half,
    /// This member’s value specifies the required architecture for an individual file.
    /// Other values are reserved and will be assigned to new machines as necessary.
    /// Processor-specific ELF names use the machine name to distinguish them. For example,
    /// the flags mentioned below use the prefix EF_; a flag named `WIDGET` for the `EM_XYZ`
    /// machine would be called `EF_XYZ_WIDGET`.
    e_machine: Elf32Half,
    /// This member identifies the object file version.
    /// The value 1 signifies the original file format; extensions will create new versions with
    /// higher numbers. The value of `EV_CURRENT`, though given as 1 above, will change as
    /// necessary to reflect the current version number.
    e_version: Elf32Word,
    /// This member gives the virtual address to which the system first transfers control, thus
    /// starting the process. If the file has no associated entry point, this member holds zero.
    e_entry: Elf32Addr,
    /// This member holds the program header table’s file offset in bytes. If the file has no
    /// program header table, this member holds zero.
    e_phoff: Elf32Off,
    /// This member holds the section header table’s file offset in bytes. If the file has no section
    /// header table, this member holds zero.
    e_shoff: Elf32Off,
    //// This member holds processor-specific flags associated with the file. Flag names take
    /// the form `EF_machine_flag`.
    e_flags: Elf32Word,
    /// This member holds the ELF header’s size in bytes.
    e_ehsize: Elf32Half,
    /// This member holds the size in bytes of one entry in the file’s program header table; all
    /// entries are the same size.
    e_phentsize: Elf32Half,
    /// This member holds the number of entries in the program header table. Thus the pro-
    /// duct of `e_phentsize` and `e_phnum` gives the table’s size in bytes. If a file has no program
    /// header table, `e_phnum` holds the value zero.
    e_phnum: Elf32Half,
    /// This member holds a section header’s size in bytes. A section header is one entry in
    /// the section header table; all entries are the same size.
    e_shentsize: Elf32Half,
    /// This member holds the number of entries in the section header table. Thus the product
    /// of e_shentsize and `e_shnum` gives the section header table’s size in bytes. If a file
    /// has no section header table, `e_shnum` holds the value zero.
    e_shnum: Elf32Half,
    /// This member holds the section header table index of the entry associated with the section
    /// name string table. If the file has no section name string table, this member holds
    /// the value `SHN_UNDEF`.
    e_shstrndx: Elf32Half,
}

const SHN_UNDEF: usize = 0;
const SHN_LORESERVE_LOPROC: usize = 0xff00;
const SHN_HIPROC: usize = 0xff1f;
const SHN_ABS: usize = 0xfff1;
const SHN_COMMON: usize = 0xfff2;
const SHN_HIRESERVE: usize = 0xffff;

const SHT_NULL: Elf32Word = 0;
const SHT_PROGBITS: Elf32Word = 1;
const SHT_SYMTAB: Elf32Word = 2;
const SHT_STRTAB: Elf32Word = 3;
const SHT_RELA: Elf32Word = 4;
const SHT_HASH: Elf32Word = 5;
const SHT_DYNAMIC: Elf32Word = 6;
const SHT_NOTE: Elf32Word = 7;
const SHT_NOBITS: Elf32Word = 8;
const SHT_REL: Elf32Word = 9;
const SHT_SHLIB: Elf32Word = 10;
const SHT_DYNSYM: Elf32Word = 11;
const SHT_LOPROC: Elf32Word = 0x70000000;
const SHT_HIPROC: Elf32Word = 0x7fffffff;
const SHT_LOUSER: Elf32Word = 0x80000000;
const SHT_HIUSER: Elf32Word = 0xffffffff;

const SHF_WRITE: Elf32Word = 0x1;
const SHF_ALLOC: Elf32Word = 0x2;
const SHF_EXECINSTR: Elf32Word = 0x4;
const SHF_MASKPROC: Elf32Word = 0xf0000000;

const SPECIAL_SECTIONS: &[(&str, Elf32Word /*type*/, Elf32Word /*flags*/)] = &[
    (".bss", SHT_NOBITS, SHF_ALLOC | SHF_WRITE),
    (".comment", SHT_PROGBITS, 0),
    (".data", SHT_PROGBITS, SHF_ALLOC | SHF_WRITE),
    (".data1", SHT_PROGBITS, SHF_ALLOC | SHF_WRITE),
    (".debug", SHT_PROGBITS, 0),
    (".dynamic", SHT_DYNAMIC, 0 /*TODO!*/),
    (".dynstr", SHT_STRTAB, SHF_ALLOC),
    (".dynsym", SHT_DYNSYM, SHF_ALLOC),
    (".fini", SHT_PROGBITS, SHF_ALLOC | SHF_EXECINSTR),
    (".got", SHT_PROGBITS, 0 /*TODO!*/),
    (".hash", SHT_HASH, SHF_ALLOC),
    (".init", SHT_PROGBITS, SHF_ALLOC | SHF_EXECINSTR),
    (".interp", SHT_PROGBITS, 0 /*TODO!*/),
    (".line", SHT_PROGBITS, 0),
    (".note", SHT_NOTE, 0),
    (".plt", SHT_PROGBITS, 0 /*TODO!*/),
    (".rodata", SHT_PROGBITS, SHF_ALLOC),
    (".rodata1", SHT_PROGBITS, SHF_ALLOC),
    (".shstrtab", SHT_STRTAB, 0),
    (".strtab", SHT_STRTAB, 0 /*TODO!*/),
    (".symtab", SHT_SYMTAB, 0 /*TODO!*/),
    (".text", SHT_PROGBITS, SHF_ALLOC | SHF_EXECINSTR),
];

#[derive(Debug)]
#[repr(C)]
pub struct Elf32Shdr {
    /// This member specifies the name of the section. Its value is an index into the section
    /// header string table section, giving the location of a null-terminated string.
    sh_name: Elf32Word,
    /// This member categorizes the section’s contents and semantics.
    sh_type: Elf32Word,
    /// Sections support 1-bit flags that describe miscellaneous attributes.
    sh_flags: Elf32Word,
    /// If the section will appear in the memory image of a process, this member gives the
    /// address at which the section’s first byte should reside. Otherwise, the member contains 0.
    sh_addr: Elf32Addr,
    /// This member’s value gives the byte offset from the beginning of the file to the first
    /// byte in the section. One section type, `SHT_NOBITS`, occupies no
    /// space in the file, and its sh_offset member locates the conceptual placement in the file.
    sh_offset: Elf32Off,
    /// This member gives the section’s size in bytes. Unless the section type is
    /// `SHT_NOBITS`, the section occupies sh_size bytes in the file. A section of type
    /// `SHT_NOBITS` may have a non-zero size, but it occupies no space in the file.
    sh_size: Elf32Word,
    /// This member holds a section header table index link, whose interpretation depends
    /// on the section type.
    sh_link: Elf32Word,
    /// This member holds extra information, whose interpretation depends on the section
    /// type. A table below describes the values.
    sh_info: Elf32Word,
    /// Some sections have address alignment constraints. For example, if a section holds a
    /// doubleword, the system must ensure doubleword alignment for the entire section.
    /// That is, the value of sh_addr must be congruent to 0, modulo the value of
    /// sh_addralign. Currently, only 0 and positive integral powers of two are allowed.
    /// Values 0 and 1 mean the section has no alignment constraints
    sh_addralign: Elf32Word,
    /// Some sections hold a table of fixed-size entries, such as a symbol table. For such a section,
    /// this member gives the size in bytes of each entry. The member contains 0 if the
    /// section does not hold a table of fixed-size entries
    sh_entsize: Elf32Word,
}

pub struct Elf64Shdr {}

pub enum Section<'a> {
    W32(&'a Elf32Shdr),
    W64(&'a Elf64),
}

pub enum BitWidth {
    W64,
    W32,
}

pub enum Endianness {
    Little,
    Big,
}

pub enum Version {
    Current,
}

#[repr(transparent)]
pub struct Elf64([u8]);

impl Elf64 {
    fn get_header(&self) -> &Elf32Ehdr {
        unsafe { &*(self.0.as_ptr() as *const _) }
    }

    pub fn program_bit_width(&self) -> Result<BitWidth> {
        match self.get_header().e_ident[EI_CLASS] {
            ELFCLASS32 => Ok(BitWidth::W32),
            ELFCLASS64 => Ok(BitWidth::W64),
            v => Err(Error::MalformedHeader(format!(
                "invalid value for program bit width '{v}'"
            ))),
        }
    }

    pub fn program_endianess(&self) -> Result<Endianness> {
        match self.get_header().e_ident[EI_DATA] {
            ELFDATA2MSB => Ok(Endianness::Big),
            ELFDATA2LSB => Ok(Endianness::Little),
            v => Err(Error::MalformedHeader(format!(
                "invalid value for program endianness '{v}'"
            ))),
        }
    }

    pub fn program_header_version(&self) -> u8 {
        self.get_header().e_ident[EI_VERSION]
    }

    pub fn program_elf_version(&self) -> Elf32Word {
        self.get_header().e_version
    }

    pub fn program_entry(&self) -> Elf32Addr {
        self.get_header().e_entry
    }

    pub fn sections(&self) {}
}

impl Debug for Elf64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&read_elf(self).map_err(|_| core::fmt::Error)?)
    }
}

pub struct ElfOwned(Box<Elf64>);

impl Debug for ElfOwned {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Deref for ElfOwned {
    type Target = Elf64;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

fn assert_elf_header(data: &[u8]) -> Result<()> {
    if data.len() < size_of::<Elf32Ehdr>() {
        return Err(Error::InsufficientSizeForHeader(data.len()));
    }
    if &data[..4] != &[0x7f, 'E' as u8, 'L' as u8, 'F' as u8] {
        return Err(Error::InvalidHeaderMagic);
    }
    Ok(())
}

pub fn parse_elf(data: &[u8]) -> Result<ElfOwned> {
    assert_elf_header(data)?;
    let elf_ptr = ptr::from_raw_parts_mut::<Elf64>(data.as_ptr() as *mut _, data.len());
    let owned = ElfOwned(unsafe { Box::from_raw(elf_ptr) });
    Ok(owned)
}

pub fn read_elf(elf: &Elf64) -> Result<String> {
    let mut output_string = String::new();
    let mut tabs = 0;
    macro_rules! write_line {
        ($($tt:tt)*) => {
            for _ in 0..tabs{
                output_string.push_str("    ");
            }
            output_string.push_str(&format!($($tt)*));
            output_string.push('\n');
        };
    }
    macro_rules! set_tabs {
        ($tabs:literal) => {
            tabs = $tabs;
        };
    }
    write_line!("header:");
    set_tabs!(1);
    write_line!(
        "magic number: {}",
        String::from_utf8_lossy(&elf.get_header().e_ident[1..=3])
    );
    write_line!(
        "bit width: {}",
        match elf.program_bit_width()? {
            BitWidth::W64 => "64",
            BitWidth::W32 => "32",
        }
    );
    write_line!(
        "endianness: {}",
        match elf.program_endianess()? {
            Endianness::Little => "little",
            Endianness::Big => "big",
        }
    );
    write_line!("ELF version: {}", elf.program_elf_version());
    write_line!("program entry: 0x{:X}", elf.program_entry());
    set_tabs!(0);
    Ok(output_string)
}
