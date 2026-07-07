//! Samsung Galaxy Buds — best-effort layout.
//!
//! Samsung uses vendor id `0x0075` and their TWS earbuds broadcast a
//! payload that community projects (BudsAssistant, gnome-shell-buds) have
//! reverse-engineered piece by piece. The exact byte offsets vary by model
//! (Buds Live vs Pro vs 2 vs 3), so the safe path is: sniff a specific
//! header, then pick out three well-known byte positions and only trust
//! them when the header matches.
//!
//! This stub returns `None` until it sees a payload that resembles the
//! documented layout; contributions with real capture samples welcome.

use super::Components;

pub fn decode(data: &[u8]) -> Option<Components> {
    // Most Galaxy Buds payloads seen in the wild start with 0x01 0x02
    // (protocol / message-type) then a model discriminator, then batteries.
    if data.len() < 12 || data[0] != 0x01 {
        return None;
    }
    // Very conservative — until we have confirmed samples we bail out.
    // Kept as a stub so real earbud logs can update the offsets without
    // touching the dispatch code.
    None
}
