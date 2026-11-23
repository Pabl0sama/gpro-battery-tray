//! Diagnostic : liste les interfaces HID Logitech et teste la communication HID++.
//! Lancer avec : cargo run --release --bin diag

use hidapi::HidApi;
use std::time::{Duration, Instant};

fn main() {
    let api = HidApi::new().expect("HidApi::new a échoué");

    println!("=== Interfaces HID Logitech (VID 046D) ===");
    for d in api.device_list().filter(|d| d.vendor_id() == 0x046D) {
        println!(
            "PID {:04X} | interface {} | usage_page {:04X} | usage {:04X} | {}",
            d.product_id(),
            d.interface_number(),
            d.usage_page(),
            d.usage(),
            d.product_string().unwrap_or("?")
        );
    }

    println!("\n=== Test HID++ sur les interfaces 0xFF00 ===");
    for d in api
        .device_list()
        .filter(|d| d.vendor_id() == 0x046D && d.usage_page() == 0xFF00)
    {
        println!(
            "\n-> PID {:04X} usage {:04X} : ouverture…",
            d.product_id(),
            d.usage()
        );
        let dev = match d.open_device(&api) {
            Ok(dev) => dev,
            Err(e) => {
                println!("   ECHEC ouverture : {e}");
                continue;
            }
        };

        for idx in 1..=6u8 {
            // IRoot.getFeature(0x1004 UNIFIED_BATTERY / 0x1001 BATTERY_VOLTAGE)
            for fid in [0x1004u16, 0x1000, 0x1001] {
                let mut msg = [0u8; 20];
                msg[0] = 0x11;
                msg[1] = idx;
                msg[2] = 0x00;
                msg[3] = 0x0A; // fonction 0 (getFeature) + swid 0xA
                msg[4..6].copy_from_slice(&fid.to_be_bytes());
                match dev.write(&msg) {
                    Err(e) => {
                        println!("   idx {idx} feat {fid:04X} : ECHEC écriture : {e}");
                        continue;
                    }
                    Ok(_) => {}
                }
                let end = Instant::now() + Duration::from_millis(800);
                let mut got = false;
                let mut buf = [0u8; 64];
                while Instant::now() < end {
                    match dev.read_timeout(&mut buf, 200) {
                        Ok(n) if n > 0 => {
                            println!(
                                "   idx {idx} feat {fid:04X} : réponse {:02X?}",
                                &buf[..n.min(12)]
                            );
                            got = true;
                            break;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            println!("   idx {idx} feat {fid:04X} : ECHEC lecture : {e}");
                            break;
                        }
                    }
                }
                if !got {
                    println!("   idx {idx} feat {fid:04X} : pas de réponse");
                }
            }
        }
    }
    println!("\nTerminé.");
}
