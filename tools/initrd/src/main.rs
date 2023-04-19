#![feature(vec_into_raw_parts)]

use std::collections::HashSet;
use std::mem::size_of;
use std::path::PathBuf;
use std::{fs, slice};

use argh::FromArgs;

#[derive(FromArgs, Debug)]
/// Utility for creating initrd.
struct Args {
    /// output file.
    #[argh(option)]
    output: PathBuf,
    /// the files to be put into the initrd.
    #[argh(positional, greedy)]
    files: Vec<PathBuf>,
}

fn main() {
    let Args { files, output } = argh::from_env();

    #[repr(C, packed)]
    struct InitrdHeader {
        entries: u64,
        /// length in bytes for the whole initrd.
        length: u64,
    }
    const MAX_NAME_LEN: usize = 32;
    #[repr(C, packed)]
    struct InitrdEntryHeader {
        start: u64,
        /// The entry's size in bytes, excluding the header.
        length: u64,
        name: [u8; MAX_NAME_LEN],
    }

    let mut entries = 0;
    let mut length = size_of::<InitrdHeader>() as u64;
    let mut header_length = length;

    let mut initrd_entry_headers = vec![];
    let mut file_vec: Vec<u8> = vec![];

    let mut names = HashSet::new();

    for file in files {
        let metadata =
            fs::metadata(&file).expect(&format!("file '{}' doesn't exist", file.display()));
        entries += 1;
        length += metadata.len();
        header_length += size_of::<InitrdEntryHeader>() as u64;
        let name = file
            .file_name()
            .expect(&format!(
                "'{}' is not a valid path to a file",
                file.display()
            ))
            .to_str()
            .expect(&format!(
                "filename in path '{}' is not valid unicode",
                file.display()
            ));
        let mut name_array = [0; 32];
        for (i, byte) in name.bytes().enumerate() {
            assert!(i < name.len(), "filename too long '{name}'");
            name_array[i] = byte;
        }
        assert!(
            !names.contains(&name_array),
            "file with the same name already exists within the initrd"
        );
        names.insert(name_array);
        file_vec.extend_from_slice(&fs::read(file).expect("failed to read file"));
        initrd_entry_headers.push(InitrdEntryHeader {
            start: u64::MAX,
            length: metadata.len(),
            name: name_array,
        })
    }
    let initrd_header = InitrdHeader { entries, length };
    let mut start_pos = header_length;
    for entry in &mut initrd_entry_headers {
        entry.start = start_pos;
        start_pos += entry.length;
    }
    let mut byte_vec = vec![];
    unsafe {
        byte_vec.extend_from_slice(slice::from_raw_parts(
            &initrd_header as *const _ as *const u8,
            size_of::<InitrdHeader>(),
        ));
        let (ptr, len, _) = initrd_entry_headers.into_raw_parts();
        if !ptr.is_null() {
            byte_vec.extend_from_slice(slice::from_raw_parts(
                ptr as *const u8,
                len * size_of::<InitrdEntryHeader>(),
            ));
        }
        byte_vec.extend_from_slice(&file_vec);
    }

    fs::write(&output, byte_vec).expect(&format!(
        "failed to write to output file {}",
        output.display()
    ))
}
