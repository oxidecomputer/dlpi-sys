use crate as dlpi;
use pretty_hex::{HexConfig, PrettyHex};
use std::io::Result;
use std::thread::spawn;

#[test]
fn test_loopback_send_recv() -> Result<()> {
    let mc = [0xff, 0xff, 0x00, 0x00, 0x00, 0x47];

    let dh_recv = dlpi::open("sim0", 0).expect("open recv");
    dlpi::bind(dh_recv, 0x4000).expect("bind recv");
    dlpi::enable_multicast(dh_recv, &mc).expect("enable multicast");

    let message = b"do you know the muffin man?";

    let t = spawn(move || {
        let mut src = [0u8; dlpi::sys::DLPI_PHYSADDR_MAX];
        let mut msg = [0; 256];
        let n = match dlpi::recv(dh_recv, &mut src, &mut msg, -1, None) {
            Ok((_, len)) => len,
            Err(e) => panic!("recv: {}", e),
        };

        let cfg = HexConfig {
            title: false,
            ascii: true,
            width: 8,
            group: 0,
            ..HexConfig::default()
        };
        println!("\n{:?}\n", (&msg[..n]).hex_conf(cfg));

        assert!(n >= message.len());
        assert_eq!(msg[..message.len()], message[..]);
    });

    let dh_send = dlpi::open("sim1", 0).expect("send");
    dlpi::bind(dh_send, 0x4000).expect("bind");

    dlpi::send(dh_send, &mc, &message[..], None).expect("send");
    t.join().expect("join recv thread");

    dlpi::close(dh_send);
    dlpi::close(dh_recv);
    Ok(())
}
