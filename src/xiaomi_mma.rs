//! Xiaomi Mi Media Air (MMA) — SPP over RFCOMM.
//!
//! Redmi/Xiaomi TWS earbuds don't broadcast per-bud battery via BLE ADV. The
//! Android app talks to them over an RFCOMM channel exposed by their SPP
//! service (vendor UUID `df21fe2c-2515-4fdb-8886-f12c4d67927c`) using the
//! MMA framing:
//!
//!     FE DC BA <opcode> <payload…> <checksum>
//!
//! At this stage we only know the header magic — the exact opcodes come
//! from either sniffing the Android app's traffic or from community repos.
//! This module ships a `probe` helper that opens the socket, writes a
//! candidate frame, and hex-dumps every byte the earbuds return so we can
//! iterate.

use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use bluer::{
    rfcomm::{SocketAddr, Stream},
    Address,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

/// Xiaomi's vendor-specific SPP UUID observed on Mi/Redmi earbuds.
pub const XIAOMI_SPP_UUID: &str = "df21fe2c-2515-4fdb-8886-f12c4d67927c";

/// MMA header magic — every frame starts with these three bytes.
pub const MMA_MAGIC: [u8; 3] = [0xFE, 0xDC, 0xBA];

/// Result of one probe attempt — the channel we ended up connecting to and
/// every byte the earbud sent back, tagged by "elapsed since write".
#[derive(Clone, Debug)]
pub struct ProbeResult {
    pub channel: u8,
    pub connect_trace: String,
    pub sent: Vec<u8>,
    pub received: Vec<(Duration, Vec<u8>)>,
}

/// Ask `sdptool records <addr>` where the Xiaomi SPP lives. Returns
/// `Some(channel)` when we find it.
///
/// Priority:
/// 1. Xiaomi's vendor-specific UUID `df21fe2c-…` if the device happens
///    to expose it in SDP (rare — usually only in the GATT service list)
/// 2. The plain SPP class id (0x1101) — Redmi Buds re-use this for MMA
pub async fn find_xiaomi_channel(address: Address) -> Option<u8> {
    let output = TokioCommand::new("sdptool")
        .args(["records", &address.to_string()])
        .output()
        .await
        .ok()?;
    let txt = String::from_utf8_lossy(&output.stdout);

    // sdptool output is a sequence of "Service ..." blocks. Split on the
    // Service RecHandle header.
    let blocks: Vec<&str> = txt.split("Service RecHandle").collect();

    let looks_like_target = |block: &str| -> bool {
        let low = block.to_lowercase();
        low.contains(XIAOMI_SPP_UUID)                // vendor SPP
            || low.contains("serial port\" (0x1101)") // plain SPP
            || low.contains("(0x1101)")              // permissive fallback
    };

    for block in blocks {
        if !looks_like_target(block) {
            continue;
        }
        // find "Channel: N" after the RFCOMM protocol line
        for line in block.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("Channel:") {
                if let Ok(ch) = rest.trim().parse::<u8>() {
                    if ch > 0 {
                        return Some(ch);
                    }
                }
            }
        }
    }
    None
}

/// Open an RFCOMM stream to the earbud. Prefers the channel we discover
/// via SDP; falls back to a small list of channels that show up in the
/// community reverse-engineering repos when SDP fails.
///
/// Returns the stream, the channel we ended up on, and a human-readable
/// trace of every attempt for debugging.
pub async fn connect(address: Address) -> Result<(Stream, u8, String)> {
    let mut trace = String::new();

    if let Some(ch) = find_xiaomi_channel(address).await {
        trace.push_str(&format!("sdptool suggests channel {ch}\n"));
        match open(address, ch).await {
            Ok(stream) => {
                trace.push_str(&format!("→ channel {ch} accepted\n"));
                return Ok((stream, ch, trace));
            }
            Err(e) => trace.push_str(&format!("→ channel {ch} refused: {e:#}\n")),
        }
    } else {
        trace.push_str("sdptool: no SPP-style channel matched, using fallback list\n");
    }

    // Fallback: try SPP channel 1 first (what most Xiaomi TWS use), then
    // channels community projects have seen in the wild.
    for candidate in [1u8, 25, 22, 24, 26, 20, 21, 23, 30, 3, 5] {
        match open(address, candidate).await {
            Ok(stream) => {
                trace.push_str(&format!("→ channel {candidate} accepted\n"));
                return Ok((stream, candidate, trace));
            }
            Err(e) => {
                trace.push_str(&format!("→ channel {candidate} refused: {e:#}\n"));
            }
        }
    }
    Err(anyhow!("no RFCOMM channel accepted the connection\n{trace}"))
}

async fn open(address: Address, channel: u8) -> Result<Stream> {
    // Use the high-level `Stream::connect` helper — the manual
    // `Socket::new()` + `.connect()` pattern in bluer 0.17 was returning
    // EALREADY (os error 114) on every attempt because the underlying
    // socket wasn't waiting for POLLOUT before checking SO_ERROR.
    // `Stream::connect` handles that pump internally.
    let addr = SocketAddr::new(address, channel);
    let stream = Stream::connect(addr)
        .await
        .with_context(|| format!("connect() on channel {channel}"))?;
    Ok(stream)
}

/// Try the given frame, then read back for up to `wait` time. Every
/// received chunk is appended to the result.
pub async fn probe(address: Address, frame: Vec<u8>, wait: Duration) -> Result<ProbeResult> {
    let (mut stream, channel, connect_trace) = connect(address).await?;
    let started = tokio::time::Instant::now();

    stream
        .write_all(&frame)
        .await
        .context("failed to write MMA probe frame")?;
    stream.flush().await.ok();

    let mut received = Vec::new();
    let deadline = started + wait;
    let mut buf = [0u8; 256];
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match timeout(remaining, stream.read(&mut buf)).await {
            Ok(Ok(0)) => break, // clean EOF
            Ok(Ok(n)) => {
                let elapsed = started.elapsed();
                received.push((elapsed, buf[..n].to_vec()));
            }
            Ok(Err(_)) | Err(_) => break,
        }
    }

    Ok(ProbeResult {
        channel,
        connect_trace,
        sent: frame,
        received,
    })
}

/// The default "hello, are you a Xiaomi earbud?" frame we send first.
///
/// Empirically the Redmi Buds 4 Active ignore whatever we send and reply
/// with their own `03 TT 00 LL <payload>` frames as soon as the RFCOMM
/// session opens, so this is mostly a "keep the connection alive" byte.
pub fn default_probe() -> Vec<u8> {
    let mut f = Vec::with_capacity(8);
    f.extend_from_slice(&MMA_MAGIC);
    f.extend_from_slice(&[0x00, 0x00, 0x01, 0x00, 0x00]);
    f
}

// ─────────────────────────────────────────────────────────────
// Xiaomi/Redmi TWS SPP frame parser
//
// Frames observed on Redmi Buds 4 Active over RFCOMM channel 5:
//
//     03 <type> <len-hi> <len-lo> <payload…>
//
//     type 0x01 — hello / version (3 bytes payload)
//     type 0x02 — a MAC-shaped 6-byte identifier
//     type 0x03 — battery: L, R, case (u8 percentages, 0xFF or >100 = n/a)
//
// Multiple frames can arrive concatenated in a single read.
// ─────────────────────────────────────────────────────────────

const FRAME_MAGIC: u8 = 0x03;
pub const FRAME_TYPE_BATTERY: u8 = 0x03;

#[derive(Clone, Debug)]
pub struct XmFrame {
    pub kind: u8,
    pub payload: Vec<u8>,
}

/// Parse zero or more back-to-back frames out of a byte buffer. Bytes
/// that don't fit the framing are skipped so a garbled start doesn't
/// throw the whole read away.
pub fn parse_frames(data: &[u8]) -> Vec<XmFrame> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 4 <= data.len() {
        if data[i] != FRAME_MAGIC {
            i += 1;
            continue;
        }
        let kind = data[i + 1];
        let len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
        if i + 4 + len > data.len() {
            // truncated tail — stop and leave the rest for a future read
            break;
        }
        out.push(XmFrame {
            kind,
            payload: data[i + 4..i + 4 + len].to_vec(),
        });
        i += 4 + len;
    }
    out
}

/// Decode a "battery" (type 0x03) payload into (left, right, case)
/// percentages. Values above 100 (typically 0x7F or 0xFF) mean "not
/// reporting" — usually the case when the buds are out of it.
pub fn parse_battery(payload: &[u8]) -> Option<(Option<u8>, Option<u8>, Option<u8>)> {
    if payload.len() < 3 {
        return None;
    }
    let read = |b: u8| -> Option<u8> {
        if b <= 100 { Some(b) } else { None }
    };
    Some((read(payload[0]), read(payload[1]), read(payload[2])))
}

/// From a fully-received probe response, extract the earbud battery
/// components in the shape used by the rest of the app.
pub fn components_from_frames(frames: &[XmFrame]) -> Option<crate::vendors::Components> {
    let batt_frame = frames.iter().find(|f| f.kind == FRAME_TYPE_BATTERY)?;
    let (l, r, c) = parse_battery(&batt_frame.payload)?;
    Some(crate::vendors::Components {
        left_battery: l,
        right_battery: r,
        case_battery: c,
        left_charging: None,
        right_charging: None,
        case_charging: None,
        in_ear_left: None,
        in_ear_right: None,
        case_open: None,
        source: "Redmi/Xiaomi TWS (SPP)",
    })
}

/// Format a `ProbeResult` for a debug log — the shape we'll show in the
/// UI while we're iterating on opcodes.
pub fn format_probe(r: &ProbeResult) -> String {
    let mut s = String::new();
    s.push_str(&r.connect_trace);
    s.push_str(&format!(
        "channel {} · sent {} bytes\n> {}\n",
        r.channel,
        r.sent.len(),
        hex::encode(&r.sent)
    ));
    if r.received.is_empty() {
        s.push_str("< (no bytes back within timeout)\n");
    } else {
        for (t, chunk) in &r.received {
            s.push_str(&format!(
                "< +{:>4} ms · {} bytes · {}\n",
                t.as_millis(),
                chunk.len(),
                hex::encode(chunk)
            ));
        }
    }
    s
}
