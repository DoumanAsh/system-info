use system_info::HostName;

#[test]
fn should_get_host_name() {
    let expected = std::process::Command::new("hostname").output().expect("Get hostname");
    let expected = expected.stdout.as_slice();
    let expected = core::str::from_utf8(expected).expect("utf-8 hostname").trim();

    println!("expected hostname={}", expected);
    let name = match HostName::get() {
        Some(name) => name,
        None => panic!("Cannot get hostname: {}", std::io::Error::last_os_error()),
    };
    assert_eq!(name.as_bytes(), expected.as_bytes());
    assert_eq!(name.as_str(), Ok(expected));
}
