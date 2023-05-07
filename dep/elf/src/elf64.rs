#![allow(dead_code)]

use core::fmt::{Debug, Display};
use core::mem::{align_of, size_of};
use core::ops::Deref;
use core::ptr;

use alloc::boxed::Box;
use alloc::slice;
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

type Elf64Half = u16;
type Elf64Word = u32;
type Elf64Sword = i32;
type Elf64Xword = u64;
type Elf64Sxword = i64;
type Elf64Addr = u64;
type Elf64Off = u64;

const EI_CLASS: usize = 4;
const EI_DATA: usize = 5;
const EI_VERSION: usize = 6;
const EI_OS_ABI: usize = 7;
const EI_ABI_VERSION: usize = 8;
const EI_NIDENT: usize = 16;

const ET_NONE: Elf64Half = 0;
const ET_REL: Elf64Half = 1;
const ET_EXEC: Elf64Half = 2;
const ET_DYN: Elf64Half = 3;
const ET_CORE: Elf64Half = 4;
const ET_LOPROC: Elf64Half = 0xff00;
const ET_HIPROC: Elf64Half = 0xffff;

const EM_NONE: Elf64Half = 0;
const EM_M32: Elf64Half = 1;
const EM_SPARC: Elf64Half = 2;
const EM_386: Elf64Half = 3;
const EM_68K: Elf64Half = 4;
const EM_88K: Elf64Half = 5;
const EM_860: Elf64Half = 7;
const EM_MIPS: Elf64Half = 8;

const EV_NONE: Elf64Word = 0;
const EV_CURRENT: Elf64Word = 1;

const ELFCLASSNONE: u8 = 0;
const ELFCLASS32: u8 = 1;
const ELFCLASS64: u8 = 2;

const ELFDATANONE: u8 = 0;
const ELFDATA2LSB: u8 = 1;
const ELFDATA2MSB: u8 = 2;

#[derive(Debug)]
#[repr(C, packed)]
pub struct Elf64Ehdr {
    /// The initial bytes mark the file as an object file and provide machine-independent data
    /// with which to decode and interpret the file’s contents.
    e_ident: [u8; EI_NIDENT],
    /// This member identifies the object file type.
    /// Although the core file contents are unspecified, type `ET_CORE` is reserved to mark the
    /// file. Values from `ET_LOPROC` through `ET_HIPROC` (inclusive) are reserved for
    /// processor-specific semantics. Other values are reserved and will be assigned to new
    /// object file types as necessary.
    e_type: Elf64Half,
    /// This member’s value specifies the required architecture for an individual file.
    /// Other values are reserved and will be assigned to new machines as necessary.
    /// Processor-specific ELF names use the machine name to distinguish them. For example,
    /// the flags mentioned below use the prefix EF_; a flag named `WIDGET` for the `EM_XYZ`
    /// machine would be called `EF_XYZ_WIDGET`.
    e_machine: Elf64Half,
    /// This member identifies the object file version.
    /// The value 1 signifies the original file format; extensions will create new versions with
    /// higher numbers. The value of `EV_CURRENT`, though given as 1 above, will change as
    /// necessary to reflect the current version number.
    e_version: Elf64Word,
    /// This member gives the virtual address to which the system first transfers control, thus
    /// starting the process. If the file has no associated entry point, this member holds zero.
    e_entry: Elf64Addr,
    /// This member holds the program header table’s file offset in bytes. If the file has no
    /// program header table, this member holds zero.
    e_phoff: Elf64Off,
    /// This member holds the section header table’s file offset in bytes. If the file has no section
    /// header table, this member holds zero.
    e_shoff: Elf64Off,
    //// This member holds processor-specific flags associated with the file. Flag names take
    /// the form `EF_machine_flag`.
    e_flags: Elf64Word,
    /// This member holds the ELF header’s size in bytes.
    e_ehsize: Elf64Half,
    /// This member holds the size in bytes of one entry in the file’s program header table; all
    /// entries are the same size.
    e_phentsize: Elf64Half,
    /// This member holds the number of entries in the program header table. Thus the pro-
    /// duct of `e_phentsize` and `e_phnum` gives the table’s size in bytes. If a file has no program
    /// header table, `e_phnum` holds the value zero.
    e_phnum: Elf64Half,
    /// This member holds a section header’s size in bytes. A section header is one entry in
    /// the section header table; all entries are the same size.
    e_shentsize: Elf64Half,
    /// This member holds the number of entries in the section header table. Thus the product
    /// of e_shentsize and `e_shnum` gives the section header table’s size in bytes. If a file
    /// has no section header table, `e_shnum` holds the value zero.
    e_shnum: Elf64Half,
    /// This member holds the section header table index of the entry associated with the section
    /// name string table. If the file has no section name string table, this member holds
    /// the value `SHN_UNDEF`.
    e_shstrndx: Elf64Half,
}
const_assert!(size_of::<Elf64Ehdr>() == 0x40);
const_assert!(align_of::<Elf64Ehdr>() == 1);

const SHN_UNDEF: usize = 0;
const SHN_LORESERVE_LOPROC: usize = 0xff00;
const SHN_HIPROC: usize = 0xff1f;
const SHN_ABS: usize = 0xfff1;
const SHN_COMMON: usize = 0xfff2;
const SHN_HIRESERVE: usize = 0xffff;

const SHT_NULL: Elf64Word = 0;
const SHT_PROGBITS: Elf64Word = 1;
const SHT_SYMTAB: Elf64Word = 2;
const SHT_STRTAB: Elf64Word = 3;
const SHT_RELA: Elf64Word = 4;
const SHT_HASH: Elf64Word = 5;
const SHT_DYNAMIC: Elf64Word = 6;
const SHT_NOTE: Elf64Word = 7;
const SHT_NOBITS: Elf64Word = 8;
const SHT_REL: Elf64Word = 9;
const SHT_SHLIB: Elf64Word = 10;
const SHT_DYNSYM: Elf64Word = 11;
const SHT_LOPROC: Elf64Word = 0x70000000;
const SHT_HIPROC: Elf64Word = 0x7fffffff;
const SHT_LOUSER: Elf64Word = 0x80000000;
const SHT_HIUSER: Elf64Word = 0xffffffff;

const SHF_WRITE: Elf64Word = 0x1;
const SHF_ALLOC: Elf64Word = 0x2;
const SHF_EXECINSTR: Elf64Word = 0x4;
const SHF_MASKPROC: Elf64Word = 0xf0000000;

const SPECIAL_SECTIONS: &[(&str, Elf64Word /*type*/, Elf64Word /*flags*/)] = &[
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
#[repr(C, packed)]
pub struct Elf64Shdr {
    /// This member specifies the name of the section. Its value is an index into the section
    /// header string table section, giving the location of a null-terminated string.
    pub sh_name: Elf64Word,
    /// This member categorizes the section’s contents and semantics.
    pub sh_type: Elf64Word,
    /// Sections support 1-bit flags that describe miscellaneous attributes.
    pub sh_flags: Elf64Xword,
    /// If the section will appear in the memory image of a process, this member gives the
    /// address at which the section’s first byte should reside. Otherwise, the member contains 0.
    pub sh_addr: Elf64Addr,
    /// This member’s value gives the byte offset from the beginning of the file to the first
    /// byte in the section. One section type, `SHT_NOBITS`, occupies no
    /// space in the file, and its sh_offset member locates the conceptual placement in the file.
    pub sh_offset: Elf64Off,
    /// This member gives the section’s size in bytes. Unless the section type is
    /// `SHT_NOBITS`, the section occupies sh_size bytes in the file. A section of type
    /// `SHT_NOBITS` may have a non-zero size, but it occupies no space in the file.
    pub sh_size: Elf64Xword,
    /// This member holds a section header table index link, whose interpretation depends
    /// on the section type.
    pub sh_link: Elf64Word,
    /// This member holds extra information, whose interpretation depends on the section
    /// type. A table below describes the values.
    pub sh_info: Elf64Word,
    /// Some sections have address alignment constraints. For example, if a section holds a
    /// doubleword, the system must ensure doubleword alignment for the entire section.
    /// That is, the value of sh_addr must be congruent to 0, modulo the value of
    /// sh_addralign. Currently, only 0 and positive integral powers of two are allowed.
    /// Values 0 and 1 mean the section has no alignment constraints.
    pub sh_addralign: Elf64Xword,
    /// Some sections hold a table of fixed-size entries, such as a symbol table. For such a section,
    /// this member gives the size in bytes of each entry. The member contains 0 if the
    /// section does not hold a table of fixed-size entries.
    pub sh_entsize: Elf64Xword,
}
const_assert!(size_of::<Elf64Shdr>() == 0x40);
const_assert!(align_of::<Elf64Shdr>() == 1);

#[derive(Debug)]
#[repr(C, packed)]
pub struct Elf64Phdr {
    /// Identifies the type of segment.
    pub p_type: Elf64Word,
    /// Contains the segment attributes. The top eight bits are reserved for processor-specific
    /// use, and the next eight bits are reserved for environment-specific use.
    pub p_flags: Elf64Word,
    /// Contains the offset, in bytes, of the segment from the beginning of the file.
    pub p_offset: Elf64Off,
    /// Contains the virtual address of the segment in memory.
    pub p_vaddr: Elf64Addr,
    /// Reserved for systems with physical addressing.
    pub p_paddr: Elf64Addr,
    /// Contains the size, in bytes, of the file image of the segment.
    pub p_filesz: Elf64Xword,
    /// Contains the size, in bytes, of the memory image of the segment.
    pub p_memsz: Elf64Xword,
    /// Specifies the alignment constraint for the segment. Must be a power of two. The values
    /// of `p_offset` and p_vaddr must be congruent modulo the alignment.
    pub p_align: Elf64Xword,
}
const_assert!(size_of::<Elf64Phdr>() == 0x38);
const_assert!(align_of::<Elf64Phdr>() == 1);

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

fn assert_elf_header(data: &[u8]) -> Result<()> {
    if data.len() < size_of::<Elf64Ehdr>() {
        return Err(Error::InsufficientSizeForHeader(data.len()));
    }
    if &data[..4] != &[0x7f, 'E' as u8, 'L' as u8, 'F' as u8] {
        return Err(Error::InvalidHeaderMagic);
    }
    Ok(())
}

impl Elf64 {
    pub fn parse_elf(data: Box<[u8]>) -> Result<Elf64Owned> {
        let len = data.len();
        let ptr = Box::into_raw(data) as *mut u8;
        assert_elf_header(unsafe { slice::from_raw_parts(ptr, len) })?;
        let elf_ptr = ptr::from_raw_parts_mut::<Elf64>(ptr as *mut _, len);
        let owned = Elf64Owned(unsafe { Box::from_raw(elf_ptr) });
        Ok(owned)
    }

    fn sections_ptr(&self) -> *const Elf64Shdr {
        (self.0.as_ptr() as u64 + self.header().e_shoff) as *const _
    }

    fn program_headers_ptr(&self) -> *const Elf64Phdr {
        (self.0.as_ptr() as u64 + self.header().e_phoff) as *const _
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.0.as_ptr(), self.0.len()) }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    pub fn header(&self) -> &Elf64Ehdr {
        unsafe { &*(self.0.as_ptr() as *const Elf64Ehdr) }
    }

    pub fn program_bit_width(&self) -> Result<BitWidth> {
        match self.header().e_ident[EI_CLASS] {
            ELFCLASS32 => Ok(BitWidth::W32),
            ELFCLASS64 => Ok(BitWidth::W64),
            v => Err(Error::MalformedHeader(format!(
                "invalid value for program bit width '{v}'"
            ))),
        }
    }

    pub fn program_endianess(&self) -> Result<Endianness> {
        match self.header().e_ident[EI_DATA] {
            ELFDATA2MSB => Ok(Endianness::Big),
            ELFDATA2LSB => Ok(Endianness::Little),
            v => Err(Error::MalformedHeader(format!(
                "invalid value for program endianness '{v}'"
            ))),
        }
    }

    pub fn program_header_version(&self) -> u8 {
        self.header().e_ident[EI_VERSION]
    }

    pub fn program_elf_version(&self) -> Elf64Word {
        self.header().e_version
    }

    pub fn program_entry(&self) -> Elf64Addr {
        self.header().e_entry
    }

    pub fn section_count(&self) -> usize {
        self.header().e_shnum as usize
    }

    pub fn sections(&self) -> &[Elf64Shdr] {
        unsafe { slice::from_raw_parts(self.sections_ptr(), self.section_count()) }
    }

    pub fn program_header_count(&self) -> usize {
        self.header().e_phnum as usize
    }

    pub fn program_headers(&self) -> &[Elf64Phdr] {
        unsafe { slice::from_raw_parts(self.program_headers_ptr(), self.program_header_count()) }
    }
}

impl Debug for Elf64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&read_elf(self).map_err(|_| core::fmt::Error)?)
    }
}

pub struct Elf64Owned(Box<Elf64>);

impl Elf64Owned {
    pub fn parse_elf(data: Box<[u8]>) -> Result<Self> {
        Elf64::parse_elf(data)
    }
}

impl Debug for Elf64Owned {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Deref for Elf64Owned {
    type Target = Elf64;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub fn parse_elf(data: Box<[u8]>) -> Result<Elf64Owned> {
    Elf64::parse_elf(data)
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
        String::from_utf8_lossy(&elf.header().e_ident[1..=3])
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
    Ok(output_string)
}
