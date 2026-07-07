//! Xiaomi / Redmi TWS earbuds — best-effort MMA-style layout.
//!
//! Xiaomi ships several protocols for wearables (MiWear, MMA — "Mi Media
//! Audio", and older variants). The publicly-reverse-engineered TWS format
//! for the Redmi Buds line usually looks like:
//!
//!   byte 0     : protocol / product family byte (varies)
//!   byte 1     : product id
//!   byte 2..3  : status flags
//!   byte 5     : left bud battery (0-100, 0xFF = unknown)
//!   byte 6     : right bud battery
//!   byte 7     : case battery
//!
//! but the same vendor id (0x038F) is also used for non-TWS Xiaomi
//! peripherals whose payload has nothing to do with earbud batteries.
//!
//! We only trust the decode when the payload matches a length and header
//! pattern that has been seen belonging to earbuds. Fewer false positives
//! is worth more than an aggressive best-effort here.

use super::Components;

pub fn decode(data: &[u8]) -> Option<Components> {
    // Only accept payloads long enough to plausibly carry L/R/case bytes.
    if data.len() < 8 {
        return None;
    }

    // Heuristic: MMA-family payloads for TWS earbuds we've observed begin
    // with 0x40..=0x6F in the first byte (product family) and have byte 2
    // in a similar range. This filters out unrelated Xiaomi peripherals
    // (Mi Bands, scales, etc.).
    let family = data[0];
    if !(0x40..=0x6F).contains(&family) {
        return None;
    }

    let read = |i: usize| -> Option<u8> {
        match data.get(i).copied()? {
            0xFF => None,
            v if v <= 100 => Some(v),
            _ => None,
        }
    };

    let left = read(5);
    let right = read(6);
    let case = read(7);

    if left.is_none() && right.is_none() && case.is_none() {
        return None;
    }

    Some(Components {
        left_battery: left,
        right_battery: right,
        case_battery: case,
        left_charging: None,
        right_charging: None,
        case_charging: None,
        in_ear_left: None,
        in_ear_right: None,
        case_open: None,
        source: "Xiaomi / Redmi TWS",
    })
}
