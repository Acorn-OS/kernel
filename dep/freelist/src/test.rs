use std::println;

use crate::{Error, FreeList, Node};

fn __assert_freelist(freelist: &FreeList, assert: &[(u64, usize)]) {
    let mut head = freelist.head;
    let mut got_values = vec![];
    while !head.is_null() {
        got_values.push(unsafe { head.read() });
        head = unsafe { (*head).next }
    }
    assert!(
        got_values.len() == assert.len(),
        "unmatching node count. Asserted values: {assert:?}, got values: {got_values:?}"
    );
    for cmp @ (cmp_start, cmp_len) in assert.iter().cloned() {
        assert!(
            got_values
                .iter()
                .find(
                    |Node {
                         next: _,
                         start,
                         len,
                     }| *start == cmp_start && *len == cmp_len
                )
                .is_some(),
            "expected node '{cmp:?}'. Asserted values: {assert:?}, got values: {got_values:?}"
        );
    }
    println!("success; asserted: {assert:?}, got: {got_values:?}");
}

macro_rules! assert_freelist {
    ($freelist:expr, $( ($start:literal, $len:literal) ),*  $(,)?) => {
        __assert_freelist($freelist, &[$( ($start, $len) ),*])
    };
}

#[test]
fn alloc_small_values() {
    let mut freelist = FreeList::new();
    freelist.push_region(0, 10).unwrap();
    assert_freelist!(&freelist, (0, 10));
    freelist.alloc::<[u8; 2]>().unwrap();
    assert_freelist!(&freelist, (2, 8));
    freelist.alloc::<u32>().unwrap();
    assert_freelist!(&freelist, (2, 2), (8, 2));
    freelist.alloc::<u8>().unwrap();
    assert_freelist!(&freelist, (2, 2), (9, 1));
    freelist.alloc::<u8>().unwrap();
    assert_freelist!(&freelist, (2, 2));
    freelist.alloc::<[u8; 2]>().unwrap();
    assert_freelist!(&freelist,);
}

#[test]
fn out_of_memory() {
    let mut freelist = FreeList::new();
    freelist.push_region(0, 10).unwrap();
    assert_freelist!(&freelist, (0, 10));
    freelist.alloc::<[u8; 10]>().unwrap();
    assert_freelist!(&freelist,);
    assert!(matches!(
        freelist.alloc::<u8>(),
        Err(Error::InsufficientSpace)
    ));
    freelist.push_region(0, 1).unwrap();
    assert_freelist!(&freelist, (0, 1));
    assert!(matches!(
        freelist.alloc::<[u8; 10]>(),
        Err(Error::InsufficientSpace)
    ));
}

#[test]
fn freeing_ascending_small_values() {
    let mut freelist = FreeList::new();
    freelist.push_region(0, 10).unwrap();
    let alloc0 = freelist.alloc::<[u8; 2]>().unwrap();
    let alloc1 = freelist.alloc::<u32>().unwrap();
    let alloc2 = freelist.alloc::<u8>().unwrap();
    let alloc3 = freelist.alloc::<u8>().unwrap();
    assert_freelist!(&freelist, (2, 2));
    unsafe {
        freelist.free(alloc0);
        assert_freelist!(&freelist, (0, 4));
        freelist.free(alloc1);
        assert_freelist!(&freelist, (0, 8));
        freelist.free(alloc2);
        assert_freelist!(&freelist, (0, 9));
        freelist.free(alloc3);
        assert_freelist!(&freelist, (0, 10));
    }
}

#[test]
fn freeing_descending_small_values() {
    let mut freelist = FreeList::new();
    freelist.push_region(0, 10).unwrap();
    let alloc0 = freelist.alloc::<[u8; 2]>().unwrap();
    let alloc1 = freelist.alloc::<u32>().unwrap();
    let alloc2 = freelist.alloc::<u8>().unwrap();
    let alloc3 = freelist.alloc::<u8>().unwrap();
    assert_freelist!(&freelist, (2, 2));
    unsafe {
        freelist.free(alloc3);
        assert_freelist!(&freelist, (2, 2), (9, 1));
        freelist.free(alloc2);
        assert_freelist!(&freelist, (2, 2), (8, 2));
        freelist.free(alloc1);
        assert_freelist!(&freelist, (2, 8));
        freelist.free(alloc0);
        assert_freelist!(&freelist, (0, 10));
    }
}

#[test]
fn freeing_random_small_values() {
    let mut freelist = FreeList::new();
    freelist.push_region(0, 4).unwrap();
    assert_freelist!(&freelist, (0, 4));
    let alloc0 = freelist.alloc::<u8>().unwrap();
    let alloc1 = freelist.alloc::<u8>().unwrap();
    let alloc2 = freelist.alloc::<u8>().unwrap();
    let alloc3 = freelist.alloc::<u8>().unwrap();
    assert_freelist!(&freelist,);
    unsafe {
        freelist.free(alloc1);
        assert_freelist!(&freelist, (1, 1));
        freelist.free(alloc3);
        assert_freelist!(&freelist, (1, 1), (3, 1));
        freelist.free(alloc2);
        assert_freelist!(&freelist, (1, 3));
        freelist.free(alloc0);
        assert_freelist!(&freelist, (0, 4));
    }
}
