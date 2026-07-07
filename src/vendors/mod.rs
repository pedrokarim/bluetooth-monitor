//! Per-vendor decoders for the proprietary BLE manufacturer_data payloads
//! that TWS earbuds broadcast alongside the standard Bluetooth profiles.
//!
//! Standard BlueZ only exposes a single `Battery1.Percentage` per device
//! (which for earbuds usually reports the lower of the two buds). The bits
//! we're interested in — per-bud battery, case battery, in-ear detection —
//! live inside the raw advertisement bytes.
//!
//! Each vendor has its own layout; this module owns the dispatch and the
//! individual decoders live in their own files.

pub mod apple;
pub mod samsung;
pub mod xiaomi;

/// What we can potentially recover from a TWS earbud advertisement.
///
/// All fields are `Option` — a vendor decoder is allowed to fill only the
/// pieces it's confident about and leave the rest as `None`.
#[derive(Clone, Debug, Default)]
pub struct Components {
    pub left_battery: Option<u8>,
    pub right_battery: Option<u8>,
    pub case_battery: Option<u8>,

    pub left_charging: Option<bool>,
    pub right_charging: Option<bool>,
    pub case_charging: Option<bool>,

    pub in_ear_left: Option<bool>,
    pub in_ear_right: Option<bool>,
    pub case_open: Option<bool>,

    /// Model / vendor label rendered in the UI header ("AirPods Pro",
    /// "Redmi Buds 4", etc). Static so it never needs an allocation for the
    /// common case.
    pub source: &'static str,
}

impl Components {
    /// `true` if at least one meaningful field is populated. A vendor
    /// decoder that returned `Some(Components::default())` should be
    /// considered a decode-failure by the caller.
    pub fn is_meaningful(&self) -> bool {
        self.left_battery.is_some()
            || self.right_battery.is_some()
            || self.case_battery.is_some()
            || self.in_ear_left.is_some()
            || self.in_ear_right.is_some()
    }
}

/// Try every registered decoder for the given `(manufacturer_id, payload)`
/// pair and return the first successful decode.
pub fn decode(manufacturer_id: u16, data: &[u8], _device_name: &str) -> Option<Components> {
    let result = match manufacturer_id {
        0x004C => apple::decode(data),
        0x0075 => samsung::decode(data),
        0x038F => xiaomi::decode(data),
        _ => None,
    };
    result.filter(Components::is_meaningful)
}

/// Best-effort decode across an entire `manufacturer_data` map — earbuds
/// occasionally announce themselves under more than one vendor ID and we
/// want the first one that yields something.
pub fn decode_map(map: &std::collections::HashMap<u16, Vec<u8>>, name: &str) -> Option<Components> {
    for (id, data) in map {
        if let Some(c) = decode(*id, data, name) {
            return Some(c);
        }
    }
    None
}
