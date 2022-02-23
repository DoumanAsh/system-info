use system_info::mem;

#[test]
fn should_get_system_mem() {
    let mem = mem::SystemMemory::new();
    println!("total={}, avail={}", mem.total, mem.avail);
    assert_ne!(mem.total, 0);
    assert_ne!(mem.avail, 0);
}
