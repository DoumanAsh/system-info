use system_info::cpu;

#[test]
fn should_get_cpu_num() {
    assert_ne!(cpu::count(), 0);
}
