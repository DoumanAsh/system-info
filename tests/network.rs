use system_info::network::Interfaces;

use core::fmt::Write;

#[test]
fn should_print_network_interfaces() {
    let interfaces = match Interfaces::new() {
        Some(interfaces) => interfaces,
        None => panic!("Cannot get interfaces data {}", std::io::Error::last_os_error()),
    };

    for interface in interfaces.iter() {
        let mut addrs_text = String::new();
        for addr in interface.addresses() {
            let _ = write!(addrs_text, "addr={} net_mask={}\n", addr, addr.net_mask());
        }

        println!("interface {:?}\n{}", interface.name(), addrs_text);
    }
}
