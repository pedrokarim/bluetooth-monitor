//! Apple's "Continuity — Proximity Pairing" advertisement layout.
//!
//! Reference implementations:
//! - OpenPods (Java) — https://github.com/adolfintel/OpenPods
//! - AirStatus (Python) — https://github.com/faissaloo/AirStatus
//!
//! Byte layout of the raw `manufacturer_data` payload (indexed under
//! vendor id `0x004C`):
//!
//!   byte  0    : 0x07  — Continuity subtype "ProximityPairing"
//!   byte  1    : 0x19  — payload length (25)
//!   bytes 2..3 : 2-byte model id (big-endian)      e.g. 0x0F20 = AirPods Pro
//!   byte  10   : status flags (bit 1 = flip L/R)
//!   byte  11   : nibbles: hi = one bud batt, lo = the other (order depends on flip)
//!   byte  12   : bits[7:4] = charging flags, bits[3:0] = case charge/status
//!                charging bit 0 = right, bit 1 = left, bit 2 = case
//!   byte  14   : bits[3:0] = case battery
//!
//! Battery nibble encoding:
//!   0..=10 → nibble * 10 % (with 10 clamped to 100 %)
//!   15     → unknown / not reporting

use super::Components;

pub fn decode(data: &[u8]) -> Option<Components> {
    if data.len() < 16 {
        return None;
    }
    if data[0] != 0x07 || data[1] != 0x19 {
        return None;
    }

    let model = u16::from_be_bytes([data[2], data[3]]);
    let name = model_name(model)?;

    let status = data[10];
    // OpenPods convention: bit 1 CLEAR means the buds are swapped in the
    // nibbles (right first). When SET, left first.
    let flip = (status & 0x02) == 0;

    let batt_byte = data[11];
    let (l_raw, r_raw) = if flip {
        (batt_byte >> 4, batt_byte & 0x0F)
    } else {
        (batt_byte & 0x0F, batt_byte >> 4)
    };
    let case_raw = data[14] & 0x0F;

    let charge = (data[12] >> 4) & 0x0F;
    let right_charging = (charge & 0x01) != 0;
    let left_charging = (charge & 0x02) != 0;
    let case_charging = (charge & 0x04) != 0;

    // Bit 3 of the status byte is documented as "case lid open" on several
    // AirPods generations; treat it as best-effort.
    let case_open = (status & 0x08) != 0;

    Some(Components {
        left_battery: nibble_to_pct(l_raw),
        right_battery: nibble_to_pct(r_raw),
        case_battery: nibble_to_pct(case_raw),
        left_charging: Some(left_charging),
        right_charging: Some(right_charging),
        case_charging: Some(case_charging),
        in_ear_left: None,
        in_ear_right: None,
        case_open: Some(case_open),
        source: name,
    })
}

fn nibble_to_pct(n: u8) -> Option<u8> {
    match n {
        0..=10 => Some((n as u16 * 10).min(100) as u8),
        _ => None,
    }
}

fn model_name(id: u16) -> Option<&'static str> {
    Some(match id {
        0x0220 => "AirPods 1st gen",
        0x0F20 => "AirPods Pro",
        0x1420 => "AirPods Max",
        0x0620 => "AirPods 2nd gen",
        0x1320 => "AirPods 3rd gen",
        0x0E20 => "Powerbeats Pro",
        0x0A20 => "Beats Solo Pro",
        0x0320 => "Powerbeats3",
        0x0B20 => "Beats Studio Buds",
        0x1220 => "Beats Studio Buds+",
        0x0D20 => "Beats Fit Pro",
        _ => return None,
    })
}
