use eframe::egui;
use egui::{Color32, Painter, Pos2, Rect, Rounding, Stroke, Style, Visuals};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeKind {
    Aurora,
    Nova,
    Command,
    Radiant,
}

impl Default for ThemeKind {
    fn default() -> Self {
        ThemeKind::Aurora
    }
}

impl ThemeKind {
    pub const ALL: [ThemeKind; 4] = [
        ThemeKind::Aurora,
        ThemeKind::Nova,
        ThemeKind::Command,
        ThemeKind::Radiant,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ThemeKind::Aurora => "Aurora",
            ThemeKind::Nova => "Nova",
            ThemeKind::Command => "Command",
            ThemeKind::Radiant => "Radiant",
        }
    }

    pub fn tagline(self) -> &'static str {
        match self {
            ThemeKind::Aurora => "Violet & warm",
            ThemeKind::Nova => "Charcoal & indigo",
            ThemeKind::Command => "Terminal green",
            ThemeKind::Radiant => "Light editorial",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum HeroFont {
    /// Inter Light — Aurora
    InterLight,
    /// Space Grotesk Bold — Nova
    SpaceGrotesk,
    /// JetBrains Mono Bold — Command
    Mono,
    /// Instrument Serif Italic — Radiant
    SerifItalic,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BgStyle {
    /// 3-stop vertical gradient
    Gradient,
    /// Solid single-color fill
    Solid,
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub kind: ThemeKind,

    pub bg_top: Color32,
    pub bg_mid: Color32,
    pub bg_bot: Color32,

    pub card: Color32,
    pub card_strong: Color32,
    pub card_outline: Color32,
    pub pill_track: Color32,

    pub text: Color32,
    pub text_muted: Color32,
    pub text_dim: Color32,

    pub teal: Color32,
    pub coral: Color32,
    pub yellow: Color32,
    pub orange: Color32,
    pub purple: Color32,
    pub pink: Color32,
    pub red: Color32,
    pub green: Color32,

    pub is_light: bool,

    // Structural differentiators
    pub bg_style: BgStyle,
    pub card_radius: f32,
    pub pill_radius: f32,
    pub chip_radius: f32,
    pub hero_font: HeroFont,
}

impl Theme {
    pub fn from_kind(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Aurora => Self::aurora(),
            ThemeKind::Nova => Self::nova(),
            ThemeKind::Command => Self::command(),
            ThemeKind::Radiant => Self::radiant(),
        }
    }

    pub fn aurora() -> Self {
        Self {
            kind: ThemeKind::Aurora,
            bg_top: rgb(0x2a, 0x1c, 0x40),
            bg_mid: rgb(0x44, 0x27, 0x5a),
            bg_bot: rgb(0x7a, 0x40, 0x6f),
            card: rgb(0x38, 0x26, 0x55),
            card_strong: rgb(0x48, 0x30, 0x6a),
            card_outline: rgb(0x5a, 0x3c, 0x7a),
            pill_track: rgb(0x2d, 0x1c, 0x45),
            text: rgb(0xf7, 0xef, 0xf6),
            text_muted: rgb(0xca, 0xbb, 0xd9),
            text_dim: rgb(0x9a, 0x88, 0xaa),
            teal: rgb(0x4c, 0xdc, 0xcf),
            coral: rgb(0xf7, 0x84, 0x73),
            yellow: rgb(0xf5, 0xc7, 0x4a),
            orange: rgb(0xf6, 0x93, 0x5c),
            purple: rgb(0xc9, 0x8c, 0xf1),
            pink: rgb(0xea, 0x76, 0xb1),
            red: rgb(0xef, 0x5a, 0x6f),
            green: rgb(0x8f, 0xe0, 0x9a),
            is_light: false,
            bg_style: BgStyle::Gradient,
            card_radius: 22.0,
            pill_radius: 999.0,
            chip_radius: 999.0,
            hero_font: HeroFont::InterLight,
        }
    }

    pub fn nova() -> Self {
        Self {
            kind: ThemeKind::Nova,
            bg_top: rgb(0x0a, 0x0a, 0x0f),
            bg_mid: rgb(0x0a, 0x0a, 0x0f),
            bg_bot: rgb(0x0a, 0x0a, 0x0f),
            card: rgb(0x14, 0x14, 0x1c),
            card_strong: rgb(0x1c, 0x1c, 0x28),
            card_outline: rgb(0x25, 0x25, 0x32),
            pill_track: rgb(0x0a, 0x0a, 0x12),
            text: rgb(0xf5, 0xf5, 0xf7),
            text_muted: rgb(0xa0, 0xa0, 0xb0),
            text_dim: rgb(0x6b, 0x6b, 0x7e),
            teal: rgb(0x22, 0xd3, 0xee),
            coral: rgb(0xf4, 0x72, 0x8c),
            yellow: rgb(0xf5, 0x9e, 0x0b),
            orange: rgb(0xfb, 0x92, 0x3c),
            purple: rgb(0xa7, 0x8b, 0xfa),
            pink: rgb(0xf4, 0x72, 0xb6),
            red: rgb(0xef, 0x44, 0x44),
            green: rgb(0x10, 0xb9, 0x81),
            is_light: false,
            bg_style: BgStyle::Solid,
            card_radius: 12.0,
            pill_radius: 8.0,
            chip_radius: 8.0,
            hero_font: HeroFont::SpaceGrotesk,
        }
    }

    pub fn command() -> Self {
        Self {
            kind: ThemeKind::Command,
            bg_top: rgb(0x0a, 0x0d, 0x0c),
            bg_mid: rgb(0x0a, 0x0d, 0x0c),
            bg_bot: rgb(0x0a, 0x0d, 0x0c),
            card: rgb(0x0f, 0x14, 0x12),
            card_strong: rgb(0x15, 0x1a, 0x1c),
            card_outline: rgb(0x2a, 0x33, 0x2d),
            pill_track: rgb(0x0d, 0x11, 0x10),
            text: rgb(0xd9, 0xe6, 0xde),
            text_muted: rgb(0x8a, 0x99, 0x90),
            text_dim: rgb(0x58, 0x65, 0x5d),
            teal: rgb(0x4a, 0xde, 0x80),
            coral: rgb(0xf5, 0xb9, 0x42),
            yellow: rgb(0xf5, 0xb9, 0x42),
            orange: rgb(0xf5, 0xb9, 0x42),
            purple: rgb(0x67, 0xe8, 0xf9),
            pink: rgb(0xd9, 0x46, 0xef),
            red: rgb(0xef, 0x44, 0x44),
            green: rgb(0x4a, 0xde, 0x80),
            is_light: false,
            bg_style: BgStyle::Solid,
            card_radius: 2.0,
            pill_radius: 0.0,
            chip_radius: 2.0,
            hero_font: HeroFont::Mono,
        }
    }

    pub fn radiant() -> Self {
        Self {
            kind: ThemeKind::Radiant,
            bg_top: rgb(0xf6, 0xef, 0xe4),
            bg_mid: rgb(0xf0, 0xe6, 0xd5),
            bg_bot: rgb(0xea, 0xd8, 0xc0),
            card: rgb(0xff, 0xff, 0xff),
            card_strong: rgb(0xfa, 0xf5, 0xea),
            card_outline: rgb(0xdb, 0xd0, 0xbf),
            pill_track: rgb(0xe8, 0xdd, 0xcc),
            text: rgb(0x19, 0x15, 0x12),
            text_muted: rgb(0x6d, 0x61, 0x56),
            text_dim: rgb(0xa0, 0x92, 0x8a),
            teal: rgb(0x6d, 0x83, 0x65),
            coral: rgb(0xc2, 0x52, 0x36),
            yellow: rgb(0xc9, 0x9a, 0x2f),
            orange: rgb(0xd6, 0x74, 0x37),
            purple: rgb(0x8b, 0x3e, 0x50),
            pink: rgb(0xa8, 0x4c, 0x64),
            red: rgb(0xa8, 0x2d, 0x2d),
            green: rgb(0x6d, 0x83, 0x65),
            is_light: true,
            bg_style: BgStyle::Gradient,
            card_radius: 18.0,
            pill_radius: 999.0,
            chip_radius: 999.0,
            hero_font: HeroFont::SerifItalic,
        }
    }

    pub fn accent_for(&self, index: usize) -> Color32 {
        let cycle = [self.teal, self.coral, self.yellow, self.purple, self.orange, self.pink];
        cycle[index % cycle.len()]
    }

    pub fn battery_color(&self, pct: u8) -> Color32 {
        if pct >= 60 {
            self.teal
        } else if pct >= 30 {
            self.yellow
        } else if pct >= 15 {
            self.orange
        } else {
            self.coral
        }
    }

    pub fn rssi_color(&self, rssi: i16) -> Color32 {
        if rssi > -50 {
            self.teal
        } else if rssi > -65 {
            self.yellow
        } else if rssi > -80 {
            self.orange
        } else {
            self.coral
        }
    }

    /// A dark color to use on top of light accent buttons.
    pub fn on_accent(&self) -> Color32 {
        if self.is_light {
            Color32::WHITE
        } else {
            Color32::from_rgb(0x1a, 0x0f, 0x26)
        }
    }
}

fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// Apply a base egui style tuned to the given theme.
pub fn apply_style(ctx: &egui::Context, theme: &Theme) {
    let mut visuals = if theme.is_light {
        Visuals::light()
    } else {
        Visuals::dark()
    };
    visuals.window_fill = theme.bg_top;
    visuals.panel_fill = theme.bg_top;
    visuals.extreme_bg_color = theme.pill_track;
    visuals.faint_bg_color = theme.card;
    visuals.override_text_color = Some(theme.text);
    visuals.hyperlink_color = theme.teal;
    visuals.selection.bg_fill = theme.teal.linear_multiply(0.35);
    visuals.selection.stroke = Stroke::new(1.0, theme.teal);

    for w in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        w.rounding = Rounding::same(999.0);
        w.expansion = 0.0;
    }
    visuals.widgets.inactive.bg_fill = theme.card;
    visuals.widgets.inactive.weak_bg_fill = theme.card;
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, theme.text);
    visuals.widgets.hovered.bg_fill = theme.card_strong;
    visuals.widgets.hovered.weak_bg_fill = theme.card_strong;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, theme.purple);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, theme.text);
    visuals.widgets.active.bg_fill = theme.card_strong;
    visuals.widgets.active.weak_bg_fill = theme.card_strong;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, theme.teal);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, theme.text);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, theme.card_outline);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, theme.text);

    visuals.window_rounding = Rounding::same(16.0);
    visuals.menu_rounding = Rounding::same(10.0);

    let mut style = Style::default();
    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(14.0, 6.0);
    style.spacing.interact_size = egui::vec2(26.0, 28.0);
    ctx.set_style(style);
}

/// Paint the window background — gradient or solid depending on theme.
pub fn paint_background(painter: &Painter, rect: Rect, theme: &Theme) {
    match theme.bg_style {
        BgStyle::Solid => {
            painter.rect_filled(rect, Rounding::ZERO, theme.bg_top);
        }
        BgStyle::Gradient => {
            let stops = [(0.0, theme.bg_top), (0.55, theme.bg_mid), (1.0, theme.bg_bot)];
            for pair in stops.windows(2) {
                let (t0, c0) = pair[0];
                let (t1, c1) = pair[1];
                let y0 = rect.min.y + rect.height() * t0;
                let y1 = rect.min.y + rect.height() * t1;
                let strip = Rect::from_min_max(Pos2::new(rect.min.x, y0), Pos2::new(rect.max.x, y1));
                gradient_rect(painter, strip, c0, c1);
            }
        }
    }
}

fn gradient_rect(painter: &Painter, rect: Rect, top: Color32, bot: Color32) {
    use egui::epaint::{Mesh, Vertex};
    let mut mesh = Mesh::default();
    let base = mesh.vertices.len() as u32;
    let uv = egui::pos2(0.0, 0.0);
    mesh.vertices.push(Vertex { pos: rect.left_top(), uv, color: top });
    mesh.vertices.push(Vertex { pos: rect.right_top(), uv, color: top });
    mesh.vertices.push(Vertex { pos: rect.left_bottom(), uv, color: bot });
    mesh.vertices.push(Vertex { pos: rect.right_bottom(), uv, color: bot });
    mesh.indices.extend_from_slice(&[base, base + 1, base + 2, base + 1, base + 3, base + 2]);
    painter.add(egui::Shape::Mesh(mesh));
}

// ─────────────────────────────────────────────────────────────
// Icons (window + tray)
// ─────────────────────────────────────────────────────────────

pub fn app_icon(size: u32) -> egui::IconData {
    let (w, h) = (size as usize, size as usize);
    let mut pixels = vec![0u8; w * h * 4];
    draw_bluetooth_glyph(&mut pixels, w, h);
    egui::IconData { rgba: pixels, width: w as u32, height: h as u32 }
}

pub fn tray_icon_argb(size: usize) -> Vec<u8> {
    let icon = app_icon(size as u32);
    let mut argb = Vec::with_capacity(icon.rgba.len());
    for chunk in icon.rgba.chunks_exact(4) {
        argb.push(chunk[3]);
        argb.push(chunk[0]);
        argb.push(chunk[1]);
        argb.push(chunk[2]);
    }
    argb
}

fn draw_bluetooth_glyph(pixels: &mut [u8], w: usize, h: usize) {
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let r_outer = (w.min(h) as f32) / 2.0 - 1.0;
    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let idx = (y * w + x) * 4;
            if dist < r_outer - 0.5 {
                let t = (dist / r_outer).clamp(0.0, 1.0);
                pixels[idx] = lerp(0xc9, 0x88, t);
                pixels[idx + 1] = lerp(0x8c, 0x40, t);
                pixels[idx + 2] = lerp(0xf1, 0xa8, t);
                pixels[idx + 3] = 255;
            }
        }
    }
    let stroke = ((w as f32) * 0.09).max(2.0) as usize;
    let cx_i = w / 2;
    let x_left = (w as f32 * 0.30) as usize;
    let x_right = (w as f32 * 0.70) as usize;
    let y_top = (h as f32 * 0.18) as usize;
    let y_bot = (h as f32 * 0.82) as usize;
    let y_mid = h / 2;
    let y_qh = (h as f32 * 0.34) as usize;
    let y_ql = (h as f32 * 0.66) as usize;
    for y in y_top..=y_bot {
        for dx in 0..stroke {
            let x = cx_i.saturating_sub(stroke / 2) + dx;
            if x < w { paint_px(pixels, w, x, y); }
        }
    }
    line(pixels, w, h, cx_i, y_top, x_right, y_qh, stroke);
    line(pixels, w, h, x_right, y_qh, x_left, y_ql, stroke);
    line(pixels, w, h, x_left, y_qh, x_right, y_ql, stroke);
    line(pixels, w, h, x_right, y_ql, cx_i, y_bot, stroke);
    line(pixels, w, h, cx_i, y_mid, x_right, y_qh, stroke);
    line(pixels, w, h, cx_i, y_mid, x_right, y_ql, stroke);
}

fn paint_px(pixels: &mut [u8], w: usize, x: usize, y: usize) {
    let idx = (y * w + x) * 4;
    if idx + 3 < pixels.len() {
        pixels[idx] = 0xff;
        pixels[idx + 1] = 0xff;
        pixels[idx + 2] = 0xff;
        pixels[idx + 3] = 255;
    }
}

fn line(pixels: &mut [u8], w: usize, h: usize, x0: usize, y0: usize, x1: usize, y1: usize, stroke: usize) {
    let (mut x0, mut y0, x1, y1) = (x0 as i32, y0 as i32, x1 as i32, y1 as i32);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let s = stroke as i32;
    loop {
        for oy in 0..s {
            for ox in 0..s {
                let px = x0 + ox - s / 2;
                let py = y0 + oy - s / 2;
                if px >= 0 && py >= 0 && (px as usize) < w && (py as usize) < h {
                    paint_px(pixels, w, px as usize, py as usize);
                }
            }
        }
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 * (1.0 - t) + b as f32 * t) as u8
}
