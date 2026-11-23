// Pas de fenêtre console en build release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Affiche le niveau de batterie d'une Logitech G Pro Wireless (dongle Lightspeed)
//! dans la zone de notification Windows. Protocole : HID++ 2.0.

use std::ffi::CString;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use hidapi::{HidApi, HidDevice};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIconBuilder};

const LOGITECH_VID: u16 = 0x046D;
const SWID: u8 = 0x0A;
const REFRESH_SECONDS: u64 = 60;

/// Table tension (mV) -> pourcentage (approximative, Li-Po).
const VOLTAGE_TABLE: [(u16, u8); 12] = [
    (4186, 100), (4067, 90), (3989, 80), (3922, 70), (3859, 60), (3811, 50),
    (3778, 40), (3751, 30), (3722, 20), (3687, 10), (3615, 5), (3500, 0),
];

type Battery = (u8, bool); // (pourcentage, en charge)

enum UserEvent {
    Battery(Option<Battery>),
    Menu(MenuEvent),
}

// ---------------------------------------------------------------------------
// HID++ bas niveau
// ---------------------------------------------------------------------------
fn request(dev: &HidDevice, devidx: u8, feat: u8, func: u8, params: &[u8]) -> Option<Vec<u8>> {
    let mut msg = [0u8; 20];
    msg[0] = 0x11;
    msg[1] = devidx;
    msg[2] = feat;
    msg[3] = (func << 4) | SWID;
    msg[4..4 + params.len()].copy_from_slice(params);
    dev.write(&msg).ok()?;

    let end = Instant::now() + Duration::from_secs(1);
    let mut buf = [0u8; 64];
    while Instant::now() < end {
        let n = dev.read_timeout(&mut buf, 200).ok()?;
        if n < 5 || buf[1] != devidx {
            continue;
        }
        // Réponse d'erreur HID++ 2.0
        if buf[2] == 0xFF && buf[3] == feat && buf[4] == msg[3] {
            return None;
        }
        if buf[2] == feat && buf[3] == msg[3] {
            return Some(buf[4..n].to_vec());
        }
    }
    None
}

fn voltage_to_percent(mv: u16) -> u8 {
    if mv >= VOLTAGE_TABLE[0].0 {
        return 100;
    }
    if mv <= VOLTAGE_TABLE[VOLTAGE_TABLE.len() - 1].0 {
        return 0;
    }
    for w in VOLTAGE_TABLE.windows(2) {
        let (v_hi, p_hi) = w[0];
        let (v_lo, p_lo) = w[1];
        if mv >= v_lo && mv <= v_hi {
            let ratio = (mv - v_lo) as f32 / (v_hi - v_lo) as f32;
            return (p_lo as f32 + ratio * (p_hi - p_lo) as f32).round() as u8;
        }
    }
    0
}

fn read_battery(dev: &HidDevice, devidx: u8) -> Option<Battery> {
    // (feature id, fonction getStatus)
    for (feature_id, func) in [(0x1004u16, 1u8), (0x1000, 0), (0x1001, 0)] {
        // IRoot.getFeature
        let r = match request(dev, devidx, 0x00, 0, &feature_id.to_be_bytes()) {
            Some(r) if !r.is_empty() && r[0] != 0 => r,
            _ => continue,
        };
        let feat_idx = r[0];
        let r = match request(dev, devidx, feat_idx, func, &[]) {
            Some(r) if r.len() >= 3 => r,
            _ => continue,
        };
        return Some(match feature_id {
            0x1004 => (r[0], matches!(r[2], 1 | 2 | 4)),          // UNIFIED_BATTERY
            0x1000 => (r[0], matches!(r[2], 1 | 2 | 3 | 4)),      // BATTERY_STATUS
            _ => {
                // BATTERY_VOLTAGE (G Pro Wireless)
                let mv = u16::from_be_bytes([r[0], r[1]]);
                (voltage_to_percent(mv), r[2] & 0x80 != 0)
            }
        });
    }
    None
}

fn get_mouse_battery(api: &HidApi) -> Option<Battery> {
    let paths: Vec<CString> = api
        .device_list()
        .filter(|d| d.vendor_id() == LOGITECH_VID && d.usage_page() == 0xFF00)
        .map(|d| d.path().to_owned())
        .collect();

    for path in paths {
        if let Ok(dev) = api.open_path(&path) {
            for idx in 1..=6u8 {
                if let Some(b) = read_battery(&dev, idx) {
                    return Some(b);
                }
            }
        }
    }
    None
}

fn worker(proxy: EventLoopProxy<UserEvent>, rx: Receiver<()>) {
    let mut api = match HidApi::new() {
        Ok(a) => a,
        Err(_) => return,
    };
    loop {
        let _ = api.refresh_devices();
        let result = get_mouse_battery(&api);
        if proxy.send_event(UserEvent::Battery(result)).is_err() {
            break;
        }
        // Attend REFRESH_SECONDS, ou une demande d'actualisation manuelle.
        if let Err(RecvTimeoutError::Disconnected) = rx.recv_timeout(Duration::from_secs(REFRESH_SECONDS)) {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// Icône : chiffres dessinés avec une police bitmap 3x5
// ---------------------------------------------------------------------------
fn glyph(c: char) -> [u8; 5] {
    match c {
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b001, 0b001, 0b001],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        _ => [0b111, 0b001, 0b011, 0b000, 0b010], // '?'
    }
}

fn make_icon(battery: Option<Battery>) -> Icon {
    const S: usize = 32;
    let mut px = vec![0u8; S * S * 4];
    for p in px.chunks_exact_mut(4) {
        p.copy_from_slice(&[25, 25, 25, 255]);
    }

    let (text, color, charging) = match battery {
        None => ("?".to_string(), [150u8, 150, 150, 255], false),
        Some((p, ch)) => {
            let c = if p > 30 {
                [60, 200, 90, 255]
            } else if p > 15 {
                [240, 160, 30, 255]
            } else {
                [230, 60, 60, 255]
            };
            (p.to_string(), c, ch)
        }
    };

    let scale = if text.len() >= 3 { 3 } else { 4 };
    let gap = 2;
    let n = text.len();
    let w = n * 3 * scale + (n - 1) * gap;
    let h = 5 * scale;
    let x0 = (S - w) / 2;
    let y0 = (26 - h) / 2;

    for (i, ch) in text.chars().enumerate() {
        let g = glyph(ch);
        let gx = x0 + i * (3 * scale + gap);
        for (row, bits) in g.iter().enumerate() {
            for col in 0..3 {
                if bits & (1 << (2 - col)) != 0 {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let x = gx + col * scale + dx;
                            let y = y0 + row * scale + dy;
                            let o = (y * S + x) * 4;
                            px[o..o + 4].copy_from_slice(&color);
                        }
                    }
                }
            }
        }
    }

    if charging {
        for y in 27..30 {
            for x in 4..28 {
                let o = (y * S + x) * 4;
                px[o..o + 4].copy_from_slice(&[70, 150, 255, 255]);
            }
        }
    }

    Icon::from_rgba(px, S as u32, S as u32).expect("icône invalide")
}

// ---------------------------------------------------------------------------
// Application
// ---------------------------------------------------------------------------
fn main() {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    {
        let p = proxy.clone();
        MenuEvent::set_event_handler(Some(move |e| {
            let _ = p.send_event(UserEvent::Menu(e));
        }));
    }

    let menu = Menu::new();
    let refresh_item = MenuItem::new("Actualiser", true, None);
    let quit_item = MenuItem::new("Quitter", true, None);
    menu.append_items(&[&refresh_item, &quit_item]).unwrap();
    let refresh_id = refresh_item.id().clone();
    let quit_id = quit_item.id().clone();
    let mut menu = Some(menu);

    let (tx, rx) = mpsc::channel::<()>();
    thread::spawn(move || worker(proxy, rx));

    let mut tray: Option<tray_icon::TrayIcon> = None;
    let mut last: Option<Battery> = None;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                tray = Some(
                    TrayIconBuilder::new()
                        .with_menu(Box::new(menu.take().unwrap()))
                        .with_tooltip("G Pro Wireless : recherche…")
                        .with_icon(make_icon(None))
                        .build()
                        .expect("impossible de créer l'icône"),
                );
            }
            Event::UserEvent(UserEvent::Battery(result)) => {
                let stale = result.is_none();
                if result.is_some() {
                    last = result;
                }
                if let Some(t) = tray.as_ref() {
                    let tooltip = match last {
                        Some((p, ch)) => {
                            let mut s = format!("G Pro Wireless : {p} %");
                            if ch {
                                s.push_str(" (en charge)");
                            }
                            if stale {
                                s.push_str(" (souris en veille ?)");
                            }
                            s
                        }
                        None => "G Pro Wireless : introuvable".to_string(),
                    };
                    let _ = t.set_icon(Some(make_icon(last)));
                    let _ = t.set_tooltip(Some(tooltip));
                }
            }
            Event::UserEvent(UserEvent::Menu(e)) => {
                if e.id == refresh_id {
                    let _ = tx.send(());
                } else if e.id == quit_id {
                    tray.take(); // retire l'icône proprement
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}
