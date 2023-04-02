use std::println;

use super::*;

#[test]
fn random_tests() {
    let mut free_lists = FreeList::new();
    free_lists.push_region(0, 100);
    println!("{free_lists:?}");
    let ptr = free_lists.alloc::<[u8; 20]>();
    println!("{free_lists:?}");
    free_lists.alloc::<[u8; 30]>();
    println!("{free_lists:?}");
    free_lists.free(ptr);
    println!("{free_lists:?}");

    println!("new scenario");

    let mut free_lists = FreeList::new();
    free_lists.push_region(0, 100);
    println!("{free_lists:?}");
    free_lists.alloc::<[u8; 20]>();
    println!("{free_lists:?}");
    let ptr = free_lists.alloc::<[u8; 30]>();
    println!("{free_lists:?}");
    free_lists.free(ptr);
    println!("{free_lists:?}");

    panic!("just panic so we print stdout");
}
