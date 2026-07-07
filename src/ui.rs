#![allow(dead_code)]

use std::sync::atomic::Ordering;

use eframe::egui::{
    self, Align, Align2, Color32, FontId, Layout, Margin, Pos2, Rect, RichText, Rounding, Sense,
    Shape, Stroke, Ui,
};

use crate::bluetooth::{BluetoothCommand, DeviceInfo, DiscoveredDevice};
use crate::fonts::fam;
use crate::theme::{self, HeroFont, Theme, ThemeKind};
use crate::AppState;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Discover,
    Detail,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DeviceFilter {
    All,
    Connected,
    Paired,
}

#[derive(Clone)]
pub struct Icons {
    pub logo: egui::TextureHandle,
    pub menu: egui::TextureHandle,
    pub back: egui::TextureHandle,
    pub refresh: egui::TextureHandle,
    pub cog: egui::TextureHandle,
    pub radar: egui::TextureHandle,
    pub chevron: egui::TextureHandle,
}

pub struct UiState {
    pub tab: Tab,
    pub filter: DeviceFilter,
    pub theme: Theme,
    pub selected_device: Option<String>,
    pub logo: Option<egui::TextureHandle>,
    pub icons: Option<Icons>,
}

impl UiState {
    pub fn new(kind: ThemeKind) -> Self {
        Self {
            tab: Tab::Dashboard,
            filter: DeviceFilter::All,
            theme: Theme::from_kind(kind),
            selected_device: None,
            logo: None,
            icons: None,
        }
    }
}

fn load_png(ctx: &egui::Context, name: &str, bytes: &[u8]) -> egui::TextureHandle {
    let img = image::load_from_memory(bytes).expect(name).to_rgba8();
    let (w, h) = (img.width() as usize, img.height() as usize);
    let color_image = egui::ColorImage::from_rgba_unmultiplied([w, h], img.as_raw());
    ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR)
}

pub fn load_logo_texture(ctx: &egui::Context) -> egui::TextureHandle {
    load_png(ctx, "bt-logo", include_bytes!("../assets/bt-logo.png"))
}

pub fn load_icons(ctx: &egui::Context) -> Icons {
    Icons {
        logo: load_png(ctx, "bt-logo", include_bytes!("../assets/bt-logo.png")),
        menu: load_png(ctx, "icon-menu", include_bytes!("../assets/icon-menu.png")),
        back: load_png(ctx, "icon-back", include_bytes!("../assets/icon-back.png")),
        refresh: load_png(
            ctx,
            "icon-refresh",
            include_bytes!("../assets/icon-refresh.png"),
        ),
        cog: load_png(ctx, "icon-cog", include_bytes!("../assets/icon-cog.png")),
        radar: load_png(
            ctx,
            "icon-radar",
            include_bytes!("../assets/icon-radar.png"),
        ),
        chevron: load_png(
            ctx,
            "icon-chevron",
            include_bytes!("../assets/icon-chevron.png"),
        ),
    }
}

// ─── Font helpers matching the mockup rhythm ──────────────────────────────
fn f_light(size: f32) -> FontId {
    FontId::new(size, fam::light())
}
fn f_reg(size: f32) -> FontId {
    FontId::new(size, fam::regular())
}
fn f_med(size: f32) -> FontId {
    FontId::new(size, fam::medium())
}
fn f_sb(size: f32) -> FontId {
    FontId::new(size, fam::semibold())
}
fn f_bold(size: f32) -> FontId {
    FontId::new(size, fam::bold())
}
fn f_mono(size: f32) -> FontId {
    FontId::new(size, fam::mono())
}

/// Theme-selected font used for hero numbers (stats, donut center, big values)
fn hero_font(theme: &Theme, size: f32) -> FontId {
    match theme.hero_font {
        HeroFont::InterLight => FontId::new(size, fam::light()),
        HeroFont::SpaceGrotesk => FontId::new(size, fam::space_grotesk()),
        HeroFont::Mono => FontId::new(size, fam::mono()),
        HeroFont::SerifItalic => FontId::new(size * 1.2, fam::instrument_italic()),
    }
}

/// Theme-selected font used for section titles ("BLUETOOTH", "DEVICES", etc)
fn title_font(theme: &Theme, size: f32) -> FontId {
    match theme.hero_font {
        HeroFont::SpaceGrotesk => FontId::new(size, fam::space_grotesk()),
        HeroFont::SerifItalic => FontId::new(size * 1.1, fam::instrument()),
        HeroFont::Mono => FontId::new(size, fam::mono()),
        HeroFont::InterLight => FontId::new(size, fam::bold()),
    }
}

pub fn render_splash(ctx: &egui::Context, ui_state: &UiState, elapsed: f32) {
    let theme = ui_state.theme;
    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            theme::paint_background(ui.painter(), rect, &theme);
            ctx.request_repaint_after(std::time::Duration::from_millis(40));

            let painter = ui.painter();
            let center = rect.center();
            let logo_center = center + egui::vec2(0.0, -32.0);

            // Three concentric orbit rings — very faint, matching the mockup
            let ring_teal = |a: u8| {
                Color32::from_rgba_unmultiplied(theme.teal.r(), theme.teal.g(), theme.teal.b(), a)
            };
            painter.circle_stroke(logo_center, 100.0, Stroke::new(1.0, ring_teal(38)));
            painter.circle_stroke(logo_center, 85.0, Stroke::new(1.0, ring_teal(26)));
            painter.circle_stroke(logo_center, 70.0, Stroke::new(1.0, ring_teal(16)));

            // Single short sweeping arc rotating around the outer ring
            let sweep_start = -std::f32::consts::FRAC_PI_2 + elapsed * 2.5;
            let sweep_end = sweep_start + 0.55; // ~30° arc
            let seg = 20;
            let pts: Vec<Pos2> = (0..=seg)
                .map(|i| {
                    let t = i as f32 / seg as f32;
                    let a = sweep_start + (sweep_end - sweep_start) * t;
                    logo_center + egui::vec2(a.cos(), a.sin()) * 100.0
                })
                .collect();
            painter.add(Shape::line(pts, Stroke::new(2.0, theme.teal)));

            // Bluetooth glyph — from embedded PNG texture, tinted to theme.text
            if let Some(tex) = ui_state.logo.as_ref() {
                let logo_size = 70.0;
                let rect = Rect::from_center_size(logo_center, egui::vec2(logo_size, logo_size));
                painter.image(
                    tex.id(),
                    rect,
                    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    theme.text,
                );
            }

            // Title "BLUETOOTH" bold + tracked, and small "MONITOR" below
            painter.text(
                center + egui::vec2(0.0, 92.0),
                Align2::CENTER_CENTER,
                "B L U E T O O T H",
                f_bold(22.0),
                theme.text,
            );
            painter.text(
                center + egui::vec2(0.0, 116.0),
                Align2::CENTER_CENTER,
                "M O N I T O R",
                f_sb(10.5),
                theme.text_muted,
            );

            // Three pulse dots, staggered
            let dots_y = center.y + 168.0;
            for i in 0..3 {
                let dx = (i as f32 - 1.0) * 14.0;
                let phase = ((elapsed * 1.4) - i as f32 * 0.2).max(0.0);
                let s = (phase.sin() * 0.5 + 0.5).powi(2);
                let alpha = (255.0 * (0.25 + 0.75 * s)) as u8;
                painter.circle_filled(Pos2::new(center.x + dx, dots_y), 4.0, ring_teal(alpha));
            }

            // Status line at fixed distance
            painter.text(
                center + egui::vec2(0.0, 210.0),
                Align2::CENTER_CENTER,
                "INITIALIZING · BLUEZ",
                f_sb(10.0),
                theme.text_dim,
            );

            // Version tucked at the very bottom
            painter.text(
                Pos2::new(center.x, rect.max.y - 30.0),
                Align2::CENTER_CENTER,
                "v0.1.0 · RUST + EGUI",
                f_sb(9.5),
                theme.text_dim,
            );
        });
}

pub fn render(ctx: &egui::Context, state: &AppState, ui_state: &mut UiState) {
    let theme = ui_state.theme;

    // ── Titlebar pinned to the top of the window ─────────────────────
    let logo_tex = ui_state.icons.as_ref().map(|i| i.logo.clone());
    egui::TopBottomPanel::top("titlebar")
        .exact_height(TITLEBAR_H)
        .show_separator_line(false)
        .frame(egui::Frame::none().fill(darken(theme.bg_top, 0.7)))
        .show(ctx, |ui| {
            titlebar(ctx, ui, &theme, logo_tex.as_ref());
        });

    // ── Status bar pinned to the bottom of the window ────────────────
    egui::TopBottomPanel::bottom("statusbar")
        .exact_height(34.0)
        .show_separator_line(false)
        .frame(
            egui::Frame::none()
                .fill(darken(theme.bg_bot, 0.55))
                .inner_margin(Margin::symmetric(24.0, 8.0)),
        )
        .show(ctx, |ui| {
            status_bar(ui, state, &theme);
        });

    // ── Everything else fills what's left ────────────────────────────
    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(ctx, |ui| {
            // Paint the gradient behind everything in the central area.
            theme::paint_background(ui.painter(), ui.max_rect(), &theme);

            // Header row
            egui::Frame::none()
                .inner_margin(Margin::symmetric(28.0, 20.0))
                .show(ui, |ui| {
                    header(ui, state, ui_state);
                });

            // Body content — fills the rest of the central panel. Everything
            // inside scrolls if the content is taller than the viewport.
            egui::Frame::none()
                .inner_margin(Margin::symmetric(28.0, 0.0))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_source(match ui_state.tab {
                            Tab::Dashboard => "body-dashboard",
                            Tab::Discover => "body-discover",
                            Tab::Detail => "body-detail",
                            Tab::Settings => "body-settings",
                        })
                        .auto_shrink([false; 2])
                        .show(ui, |ui| match ui_state.tab {
                            Tab::Dashboard => dashboard(ui, state, ui_state),
                            Tab::Discover => discover(ui, state, ui_state),
                            Tab::Detail => detail(ui, state, ui_state),
                            Tab::Settings => settings(ui, state, ui_state),
                        });
                });
        });
}

fn darken(c: Color32, factor: f32) -> Color32 {
    Color32::from_rgb(
        (c.r() as f32 * factor).min(255.0) as u8,
        (c.g() as f32 * factor).min(255.0) as u8,
        (c.b() as f32 * factor).min(255.0) as u8,
    )
}

// ─────────────────────────────────────────────────────────────
// Custom title bar (client-side decoration)
// ─────────────────────────────────────────────────────────────

const TITLEBAR_H: f32 = 32.0;
const TITLEBAR_BTN_W: f32 = 46.0;

fn titlebar(ctx: &egui::Context, ui: &mut Ui, theme: &Theme, logo: Option<&egui::TextureHandle>) {
    // Container is a TopBottomPanel that already fills its background —
    // we just allocate the same rect so we can paint text and interact
    // widgets against known coordinates.
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), TITLEBAR_H), Sense::hover());
    ui.painter().line_segment(
        [
            egui::pos2(rect.min.x, rect.max.y - 0.5),
            egui::pos2(rect.max.x, rect.max.y - 0.5),
        ],
        Stroke::new(1.0, theme.card_outline),
    );

    // Left: logo + wordmark. Logo is the embedded Bluetooth glyph tinted
    // to teal so it reads as an identity mark, not a passive icon.
    let mut cursor_x = rect.min.x + 12.0;
    if let Some(tex) = logo {
        let size = 16.0f32;
        let logo_rect = Rect::from_center_size(
            egui::pos2(cursor_x + size / 2.0, rect.center().y),
            egui::vec2(size, size),
        );
        ui.painter().image(
            tex.id(),
            logo_rect,
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            theme.teal,
        );
        cursor_x = logo_rect.max.x + 8.0;
    }
    ui.painter().text(
        egui::pos2(cursor_x, rect.center().y),
        Align2::LEFT_CENTER,
        "BLUETOOTH MONITOR",
        f_sb(10.5),
        theme.text_muted,
    );

    // Right-anchored window control buttons: min / max·restore / close
    let close_rect = Rect::from_min_size(
        egui::pos2(rect.max.x - TITLEBAR_BTN_W, rect.min.y),
        egui::vec2(TITLEBAR_BTN_W, TITLEBAR_H),
    );
    let max_rect = Rect::from_min_size(
        egui::pos2(rect.max.x - TITLEBAR_BTN_W * 2.0, rect.min.y),
        egui::vec2(TITLEBAR_BTN_W, TITLEBAR_H),
    );
    let min_rect = Rect::from_min_size(
        egui::pos2(rect.max.x - TITLEBAR_BTN_W * 3.0, rect.min.y),
        egui::vec2(TITLEBAR_BTN_W, TITLEBAR_H),
    );

    let is_maximized = ctx
        .input(|i| i.viewport().maximized)
        .unwrap_or(false);

    if titlebar_button(ui, min_rect, TitleGlyph::Min, theme, false).clicked() {
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
    }
    let max_glyph = if is_maximized {
        TitleGlyph::Restore
    } else {
        TitleGlyph::Max
    };
    if titlebar_button(ui, max_rect, max_glyph, theme, false).clicked() {
        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
    }
    if titlebar_button(ui, close_rect, TitleGlyph::Close, theme, true).clicked() {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    // Drag area — everything between the title text and the window controls.
    let drag_rect = Rect::from_min_max(
        egui::pos2(rect.min.x, rect.min.y),
        egui::pos2(min_rect.min.x, rect.max.y),
    );
    let drag_resp = ui.interact(drag_rect, egui::Id::new("titlebar_drag"), Sense::click_and_drag());
    if drag_resp.double_clicked() {
        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
    } else if drag_resp.drag_started_by(egui::PointerButton::Primary) {
        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
}

#[derive(Copy, Clone)]
enum TitleGlyph {
    Min,
    Max,
    Restore,
    Close,
}

fn titlebar_button(ui: &mut Ui, rect: Rect, glyph: TitleGlyph, theme: &Theme, danger: bool) -> egui::Response {
    let id = ui.id().with(("titlebar", rect.min.x as i32));
    let resp = ui.interact(rect, id, Sense::click());
    let painter = ui.painter();

    let hover_fill = if danger {
        Color32::from_rgb(0xd8, 0x35, 0x50)
    } else {
        theme.card_strong
    };
    if resp.hovered() {
        painter.rect_filled(rect, Rounding::ZERO, hover_fill);
    }

    let fg = if resp.hovered() {
        theme.text
    } else {
        theme.text_muted
    };
    let stroke = Stroke::new(1.4, fg);
    let center = rect.center();
    let s = 5.0; // half-size of the glyph

    match glyph {
        TitleGlyph::Min => {
            painter.line_segment(
                [egui::pos2(center.x - s, center.y + 2.0), egui::pos2(center.x + s, center.y + 2.0)],
                stroke,
            );
        }
        TitleGlyph::Max => {
            let r = Rect::from_center_size(center, egui::vec2(s * 2.0, s * 2.0));
            painter.rect_stroke(r, Rounding::ZERO, stroke);
        }
        TitleGlyph::Restore => {
            // Two offset squares indicating "unmaximise"
            let a = Rect::from_center_size(
                center + egui::vec2(1.5, -1.5),
                egui::vec2(s * 2.0, s * 2.0),
            );
            let b = Rect::from_center_size(
                center + egui::vec2(-1.5, 1.5),
                egui::vec2(s * 2.0, s * 2.0),
            );
            painter.rect_stroke(a, Rounding::ZERO, stroke);
            painter.rect_stroke(b, Rounding::ZERO, stroke);
        }
        TitleGlyph::Close => {
            painter.line_segment(
                [egui::pos2(center.x - s, center.y - s), egui::pos2(center.x + s, center.y + s)],
                stroke,
            );
            painter.line_segment(
                [egui::pos2(center.x - s, center.y + s), egui::pos2(center.x + s, center.y - s)],
                stroke,
            );
        }
    }
    resp
}

fn status_bar(ui: &mut Ui, state: &AppState, theme: &Theme) {
    ui.horizontal(|ui| {
        let powered = state.adapter_powered.load(Ordering::Relaxed);
        let (color, label) = if powered {
            (theme.teal, "Ready")
        } else {
            (theme.coral, "Adapter off")
        };
        let (dot, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), Sense::hover());
        ui.painter().circle_filled(dot.center(), 4.0, color);
        ui.label(
            RichText::new(label)
                .font(f_reg(11.0))
                .color(theme.text_muted),
        );

        ui.add_space(10.0);
        ui.label(RichText::new("·").color(theme.text_dim));
        ui.add_space(10.0);

        let n_conn = state
            .devices
            .lock()
            .unwrap()
            .iter()
            .filter(|d| d.connected)
            .count();
        let n_paired = state
            .devices
            .lock()
            .unwrap()
            .iter()
            .filter(|d| d.paired)
            .count();
        ui.label(
            RichText::new(format!("{n_conn} connected · {n_paired} paired"))
                .font(f_reg(11.0))
                .color(theme.text_dim),
        );

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if let Some(err) = state.last_error.lock().unwrap().as_ref() {
                ui.label(
                    RichText::new(format!("⚠ {err}"))
                        .font(f_reg(11.0))
                        .color(theme.coral),
                );
            } else {
                let secs = state.config.lock().unwrap().refresh_interval_secs;
                ui.label(
                    RichText::new(format!("Auto-refresh {secs}s"))
                        .font(f_mono(10.5))
                        .color(theme.text_dim),
                );
            }
        });
    });
}

// ─────────────────────────────────────────────────────────────
// Header
// ─────────────────────────────────────────────────────────────

fn header(ui: &mut Ui, state: &AppState, ui_state: &mut UiState) {
    let theme = ui_state.theme;
    let icons = ui_state.icons.clone();
    ui.horizontal(|ui| {
        let is_deep = matches!(ui_state.tab, Tab::Detail | Tab::Settings | Tab::Discover);
        if is_deep {
            // Back-arrow chip only appears on child screens. On Dashboard the
            // cog + radar chips on the right already own navigation, so a
            // second left-side chip that also opens Settings is just noise.
            let tex = icons.as_ref().map(|i| &i.back);
            if png_chip(ui, tex, &theme, false).clicked() {
                ui_state.tab = Tab::Dashboard;
            }
            ui.add_space(14.0);
        }

        ui.vertical(|ui| {
            let title = match ui_state.tab {
                Tab::Dashboard => "BLUETOOTH",
                Tab::Discover => "DISCOVER",
                Tab::Detail => "DEVICE",
                Tab::Settings => "SETTINGS",
            };
            ui.label(
                RichText::new(title)
                    .font(title_font(&theme, 22.0))
                    .color(theme.text),
            );

            let adapter = state.adapter_name.lock().unwrap().clone();
            let n_known = state.devices.lock().unwrap().len();
            let sub = match ui_state.tab {
                Tab::Dashboard => match adapter {
                    Some(a) => format!("{} · {} KNOWN", a.to_uppercase(), n_known),
                    None => "HCI · —".into(),
                },
                Tab::Discover => {
                    let scanning = state.scanning.load(Ordering::Relaxed);
                    if scanning {
                        let elapsed = state
                            .scan_started
                            .lock()
                            .unwrap()
                            .map(|t| t.elapsed().as_secs())
                            .unwrap_or(0);
                        let found = state.nearby.lock().unwrap().len();
                        format!("SCANNING · {elapsed}s · {found} FOUND")
                    } else {
                        "IDLE · TAP RADAR TO SCAN".into()
                    }
                }
                Tab::Detail => {
                    let addr = ui_state.selected_device.clone().unwrap_or_default();
                    format!("{}", addr.to_uppercase())
                }
                Tab::Settings => "PREFERENCES".into(),
            };
            ui.label(RichText::new(sub).font(f_sb(10.5)).color(theme.text_dim));
        });

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let spinning = state
                .last_refresh
                .lock()
                .unwrap()
                .map(|t| t.elapsed().as_millis() < 400)
                .unwrap_or(false);
            let tex_refresh = icons.as_ref().map(|i| &i.refresh);
            if png_chip(ui, tex_refresh, &theme, spinning).clicked() {
                let _ = state.cmd_tx.send(BluetoothCommand::Refresh);
            }
            ui.add_space(6.0);
            let tex_cog = icons.as_ref().map(|i| &i.cog);
            if png_chip(ui, tex_cog, &theme, false).clicked() {
                ui_state.tab = Tab::Settings;
            }
            ui.add_space(6.0);
            let tex_radar = icons.as_ref().map(|i| &i.radar);
            if png_chip(ui, tex_radar, &theme, false).clicked() {
                ui_state.tab = Tab::Discover;
            }
        });
    });
}

/// Round chip that paints a PNG icon centered inside. Tinted to accent when
/// `spinning` (also rotates the texture for the refresh animation).
fn png_chip(
    ui: &mut Ui,
    tex: Option<&egui::TextureHandle>,
    theme: &Theme,
    spinning: bool,
) -> egui::Response {
    let size = egui::vec2(36.0, 36.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();

    let fill = if resp.hovered() {
        theme.card_strong
    } else {
        theme.card
    };
    let tint = if resp.hovered() || spinning {
        theme.teal
    } else {
        theme.text_muted
    };
    let radius = theme.chip_radius.min(rect.width() / 2.0);
    painter.rect_filled(rect, Rounding::same(radius), fill);
    painter.rect_stroke(
        rect,
        Rounding::same(radius),
        Stroke::new(1.0, theme.card_outline),
    );

    if let Some(t) = tex {
        let icon_size = 20.0f32;
        let center = rect.center();

        if spinning {
            let angle = (ui.input(|i| i.time) * 6.0) as f32;
            let (s, c) = angle.sin_cos();
            let hs = icon_size / 2.0;
            // Rotate the 4 corners around center
            let corners = [(-hs, -hs), (hs, -hs), (hs, hs), (-hs, hs)];
            let mut mesh = egui::epaint::Mesh::with_texture(t.id());
            let uvs = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
            for (i, (x, y)) in corners.iter().enumerate() {
                let rx = x * c - y * s;
                let ry = x * s + y * c;
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: center + egui::vec2(rx, ry),
                    uv: egui::pos2(uvs[i].0, uvs[i].1),
                    color: tint,
                });
            }
            mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
            painter.add(egui::Shape::Mesh(mesh));
            ui.ctx().request_repaint();
        } else {
            let icon_rect = Rect::from_center_size(center, egui::vec2(icon_size, icon_size));
            painter.image(
                t.id(),
                icon_rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                tint,
            );
        }
    }
    resp
}

fn refresh_chip(ui: &mut Ui, theme: &Theme, spinning: bool) -> egui::Response {
    let size = egui::vec2(36.0, 36.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    let fill = if resp.hovered() {
        theme.card_strong
    } else {
        theme.card
    };
    let fg = if resp.hovered() || spinning {
        theme.teal
    } else {
        theme.text_muted
    };
    painter.circle_filled(rect.center(), 18.0, fill);
    painter.circle_stroke(rect.center(), 18.0, Stroke::new(1.0, theme.card_outline));

    let angle = if spinning {
        (ui.input(|i| i.time) * 6.0) as f32
    } else {
        0.0
    };
    if spinning {
        ui.ctx().request_repaint();
    }
    draw_refresh_arc(painter, rect.center(), 14.0, fg, angle);
    resp
}

fn draw_refresh_arc(
    painter: &egui::Painter,
    center: Pos2,
    size: f32,
    color: Color32,
    extra_rot: f32,
) {
    let stroke = Stroke::new(1.8, color);
    let h = size / 2.0;
    let r = h * 0.75;
    let start = -std::f32::consts::FRAC_PI_2 + extra_rot;
    let end = start + std::f32::consts::PI * 1.5;
    let seg = 32;
    let pts: Vec<Pos2> = (0..=seg)
        .map(|i| {
            let t = i as f32 / seg as f32;
            let a = start + (end - start) * t;
            center + egui::vec2(a.cos(), a.sin()) * r
        })
        .collect();
    painter.add(egui::Shape::line(pts.clone(), stroke));
    let end_pt = *pts.last().unwrap();
    let a = end;
    let dir = egui::vec2(a.cos(), a.sin());
    let tan = egui::vec2(-a.sin(), a.cos());
    painter.line_segment([end_pt, end_pt + tan * 4.0 - dir * 3.0], stroke);
    painter.line_segment([end_pt, end_pt + tan * 4.0 + dir * 3.0], stroke);
}

#[derive(Copy, Clone)]
enum ChipIcon {
    Menu,
    Back,
    Refresh,
    Cog,
    Radar,
}

fn icon_chip(ui: &mut Ui, icon: ChipIcon, theme: &Theme) -> egui::Response {
    let size = egui::vec2(36.0, 36.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();

    let fill = if resp.hovered() {
        theme.card_strong
    } else {
        theme.card
    };
    let fg = if resp.hovered() {
        theme.teal
    } else {
        theme.text_muted
    };
    painter.circle_filled(rect.center(), 18.0, fill);
    painter.circle_stroke(rect.center(), 18.0, Stroke::new(1.0, theme.card_outline));

    draw_icon(painter, rect.center(), 14.0, fg, icon);
    resp
}

fn draw_icon(painter: &egui::Painter, center: Pos2, size: f32, color: Color32, icon: ChipIcon) {
    let stroke = Stroke::new(1.8, color);
    let h = size / 2.0;
    match icon {
        ChipIcon::Menu => {
            for dy in [-h * 0.55, 0.0, h * 0.55] {
                painter.line_segment(
                    [
                        Pos2::new(center.x - h * 0.85, center.y + dy),
                        Pos2::new(center.x + h * 0.85, center.y + dy),
                    ],
                    stroke,
                );
            }
        }
        ChipIcon::Back => {
            let tip = Pos2::new(center.x - h * 0.6, center.y);
            let top = Pos2::new(center.x + h * 0.2, center.y - h * 0.7);
            let bot = Pos2::new(center.x + h * 0.2, center.y + h * 0.7);
            painter.line_segment([top, tip], stroke);
            painter.line_segment([tip, bot], stroke);
            painter.line_segment([tip, Pos2::new(center.x + h * 0.85, center.y)], stroke);
        }
        ChipIcon::Refresh => {
            // 3/4 arc + arrow head
            let r = h * 0.75;
            let start = -std::f32::consts::FRAC_PI_2;
            let end = start + std::f32::consts::PI * 1.5;
            let seg = 32;
            let pts: Vec<Pos2> = (0..=seg)
                .map(|i| {
                    let t = i as f32 / seg as f32;
                    let a = start + (end - start) * t;
                    center + egui::vec2(a.cos(), a.sin()) * r
                })
                .collect();
            painter.add(egui::Shape::line(pts.clone(), stroke));
            // Arrow head at end of arc pointing outward
            let end_pt = *pts.last().unwrap();
            let a = end;
            let dir = egui::vec2(a.cos(), a.sin());
            let tan = egui::vec2(-a.sin(), a.cos()); // tangent CCW
            let head_a = end_pt + tan * 4.0 - dir * 3.0;
            let head_b = end_pt + tan * 4.0 + dir * 3.0;
            painter.line_segment([end_pt, head_a], stroke);
            painter.line_segment([end_pt, head_b], stroke);
        }
        ChipIcon::Radar => {
            // Concentric arcs + center dot
            let r_out = h * 0.85;
            let r_mid = h * 0.55;
            let r_in = h * 0.25;
            painter.circle_stroke(center, r_out, Stroke::new(1.5, color));
            painter.circle_stroke(center, r_mid, Stroke::new(1.2, color.linear_multiply(0.7)));
            painter.circle_stroke(center, r_in, Stroke::new(1.0, color.linear_multiply(0.5)));
            painter.circle_filled(center, 1.6, color);
        }
        ChipIcon::Cog => {
            // Two horizontal sliders with knobs (settings/tune)
            let w = h * 0.85;
            let ys = [-h * 0.35, h * 0.35];
            let knob_xs = [-w * 0.25, w * 0.30];
            let bar_stroke = Stroke::new(1.6, color);
            for (i, dy) in ys.iter().enumerate() {
                let y = center.y + dy;
                painter.line_segment(
                    [Pos2::new(center.x - w, y), Pos2::new(center.x + w, y)],
                    bar_stroke,
                );
                let knob = Pos2::new(center.x + knob_xs[i], y);
                painter.circle_filled(knob, 3.2, color);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Dashboard
// ─────────────────────────────────────────────────────────────

fn dashboard(ui: &mut Ui, state: &AppState, ui_state: &mut UiState) {
    let theme = ui_state.theme;
    pill_tabs(ui, &mut ui_state.filter, &theme);
    ui.add_space(22.0);

    let devices = state.devices.lock().unwrap().clone();
    stat_strip(ui, &devices, &theme);
    ui.add_space(24.0);

    let filter = ui_state.filter;
    two_column(ui, &devices, filter, state, &theme, ui_state);
}

fn pill_tabs(ui: &mut Ui, current: &mut DeviceFilter, theme: &Theme) {
    let options = [
        (DeviceFilter::All, "ALL"),
        (DeviceFilter::Connected, "CONNECTED"),
        (DeviceFilter::Paired, "PAIRED"),
    ];
    let width = 380.0;
    let height = 42.0;

    ui.horizontal(|ui| {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), Sense::hover());
            ui.painter()
                .rect_filled(rect, Rounding::same(theme.pill_radius), theme.pill_track);
            ui.painter().rect_stroke(
                rect,
                Rounding::same(theme.pill_radius),
                Stroke::new(1.0, theme.card_outline),
            );

            let seg_w = width / options.len() as f32;
            for (i, (tab, label)) in options.iter().enumerate() {
                let x = rect.min.x + seg_w * i as f32;
                let seg_rect =
                    Rect::from_min_size(Pos2::new(x, rect.min.y), egui::vec2(seg_w, height));
                let resp = ui.interact(seg_rect, ui.id().with(("pill", i)), Sense::click());
                let selected = *tab == *current;
                if resp.clicked() {
                    *current = *tab;
                }
                if selected {
                    let inner = seg_rect.shrink(4.0);
                    let inner_r = (theme.pill_radius - 4.0).max(0.0);
                    ui.painter()
                        .rect_filled(inner, Rounding::same(inner_r), theme.card_strong);
                    ui.painter().rect_stroke(
                        inner,
                        Rounding::same(inner_r),
                        Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 15)),
                    );
                }
                let color = if selected {
                    theme.text
                } else if resp.hovered() {
                    theme.text_muted
                } else {
                    theme.text_dim
                };
                let font = if selected { f_sb(11.5) } else { f_sb(11.5) };
                ui.painter()
                    .text(seg_rect.center(), Align2::CENTER_CENTER, label, font, color);
            }
        });
    });
}

fn stat_strip(ui: &mut Ui, devices: &[DeviceInfo], theme: &Theme) {
    let connected = devices.iter().filter(|d| d.connected).count();
    let paired = devices.iter().filter(|d| d.paired).count();
    let total = devices.len();

    let batteries: Vec<u8> = devices
        .iter()
        .filter(|d| d.connected)
        .filter_map(|d| d.battery)
        .collect();
    let battery_avg = if batteries.is_empty() {
        None
    } else {
        Some((batteries.iter().map(|b| *b as u32).sum::<u32>() / batteries.len() as u32) as u8)
    };
    let rssis: Vec<i16> = devices
        .iter()
        .filter(|d| d.connected)
        .filter_map(|d| d.rssi)
        .collect();
    let rssi_avg = if rssis.is_empty() {
        None
    } else {
        Some((rssis.iter().map(|r| *r as i32).sum::<i32>() / rssis.len() as i32) as i16)
    };

    ui.horizontal(|ui| {
        stat_block(
            ui,
            "CONNECTED",
            &connected.to_string(),
            &format!("of {total} devices"),
            theme.teal,
            theme,
        );
        ui.add_space(36.0);
        stat_block(
            ui,
            "PAIRED",
            &paired.to_string(),
            "known devices",
            theme.purple,
            theme,
        );
        ui.add_space(36.0);
        match battery_avg {
            Some(b) => stat_block(
                ui,
                "BATTERY AVG",
                &format!("{b}%"),
                &format!("across {} rings", batteries.len()),
                theme.battery_color(b),
                theme,
            ),
            None => stat_block(ui, "BATTERY AVG", "—", "no readings", theme.text_dim, theme),
        }
        ui.add_space(36.0);
        match rssi_avg {
            Some(r) => stat_block(
                ui,
                "SIGNAL AVG",
                &format!("{r}"),
                "dBm across",
                theme.rssi_color(r),
                theme,
            ),
            None => stat_block(ui, "SIGNAL AVG", "—", "no readings", theme.text_dim, theme),
        }
    });
}

fn stat_block(
    ui: &mut Ui,
    label: &str,
    value: &str,
    caption: &str,
    accent: Color32,
    theme: &Theme,
) {
    ui.vertical(|ui| {
        // Micro-label
        ui.label(RichText::new(label).font(f_sb(10.0)).color(theme.text_dim));
        ui.add_space(4.0);
        // Hero number — themed font family
        ui.label(
            RichText::new(value)
                .font(hero_font(theme, 40.0))
                .color(accent),
        );
        ui.label(
            RichText::new(caption)
                .font(f_reg(10.5))
                .color(theme.text_dim),
        );
    });
}

// ─────────────────────────────────────────────────────────────
// Two columns
// ─────────────────────────────────────────────────────────────

fn two_column(
    ui: &mut Ui,
    devices: &[DeviceInfo],
    filter: DeviceFilter,
    state: &AppState,
    theme: &Theme,
    ui_state: &mut UiState,
) {
    let filtered = filter_devices(devices, filter);
    let connected: Vec<&DeviceInfo> = devices.iter().filter(|d| d.connected).collect();
    let avail_h = ui.available_height().max(360.0);

    ui.columns(2, |cols| {
        cols[0].with_layout(Layout::top_down(Align::Center), |ui| {
            donut_card(ui, &connected, theme, avail_h);
        });
        cols[1].with_layout(Layout::top_down(Align::Min), |ui| {
            device_list_card(
                ui,
                &filtered,
                devices.len(),
                state,
                theme,
                avail_h,
                ui_state,
            );
        });
    });
}

fn filter_devices(devices: &[DeviceInfo], filter: DeviceFilter) -> Vec<DeviceInfo> {
    devices
        .iter()
        .filter(|d| match filter {
            DeviceFilter::All => true,
            DeviceFilter::Connected => d.connected,
            DeviceFilter::Paired => d.paired,
        })
        .cloned()
        .collect()
}

// ─────────────────────────────────────────────────────────────
// Donut card
// ─────────────────────────────────────────────────────────────

fn donut_card(ui: &mut Ui, connected: &[&DeviceInfo], theme: &Theme, _target_h: f32) {
    egui::Frame::none()
        .fill(theme.card)
        .stroke(Stroke::new(1.0, theme.card_outline))
        .rounding(Rounding::same(theme.card_radius))
        .inner_margin(Margin::symmetric(22.0, 22.0))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("BATTERY RINGS")
                            .font(f_sb(10.0))
                            .color(theme.text_muted),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        live_badge(ui, theme);
                    });
                });
                ui.add_space(12.0);

                let size = 240.0f32;
                let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
                paint_donut(ui, rect, connected, theme);

                ui.add_space(18.0);
                if connected.is_empty() {
                    ui.label(
                        RichText::new("No connected devices")
                            .font(f_reg(11.0))
                            .color(theme.text_dim),
                    );
                } else {
                    for (i, d) in connected.iter().enumerate() {
                        legend_row(ui, theme.accent_for(i), &d.name, d.battery, theme);
                    }
                }
            });
        });
}

fn paint_donut(ui: &Ui, rect: Rect, connected: &[&DeviceInfo], theme: &Theme) {
    let painter = ui.painter();
    let center = rect.center();
    let ring_w = 10.0;
    let gap = 5.0;
    let outer = rect.width().min(rect.height()) / 2.0 - 4.0;

    for (i, d) in connected.iter().enumerate() {
        let r = outer - i as f32 * (ring_w + gap);
        if r < ring_w * 1.5 {
            break;
        }
        let color = theme.accent_for(i);
        painter.add(Shape::circle_stroke(
            center,
            r,
            Stroke::new(ring_w, theme.pill_track),
        ));
        let pct = d.battery.unwrap_or(100) as f32 / 100.0;
        let start = -std::f32::consts::FRAC_PI_2;
        let end = start + pct * std::f32::consts::TAU;
        arc_stroke(painter, center, r, start, end, Stroke::new(ring_w, color));
    }

    let count = connected.len();
    painter.text(
        center - egui::vec2(0.0, 10.0),
        Align2::CENTER_CENTER,
        count.to_string(),
        hero_font(theme, 50.0),
        theme.text,
    );
    painter.text(
        center + egui::vec2(0.0, 26.0),
        Align2::CENTER_CENTER,
        "CONNECTED",
        f_sb(10.0),
        theme.text_dim,
    );
}

fn arc_stroke(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    start: f32,
    end: f32,
    stroke: Stroke,
) {
    let segments = 96;
    let points: Vec<Pos2> = (0..=segments)
        .map(|i| {
            let t = i as f32 / segments as f32;
            let a = start + (end - start) * t;
            center + egui::vec2(a.cos(), a.sin()) * radius
        })
        .collect();
    painter.add(Shape::line(points, stroke));
}

fn legend_row(ui: &mut Ui, color: Color32, name: &str, battery: Option<u8>, theme: &Theme) {
    ui.horizontal(|ui| {
        let (dot, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), Sense::hover());
        ui.painter().circle_filled(dot.center(), 5.0, color);
        ui.label(
            RichText::new(name)
                .font(f_reg(12.0))
                .color(theme.text_muted),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let text = match battery {
                Some(b) => format!("{b}%"),
                None => "—".into(),
            };
            ui.label(
                RichText::new(text).font(f_sb(12.0)).color(
                    battery
                        .map(|b| theme.battery_color(b))
                        .unwrap_or(theme.text_dim),
                ),
            );
        });
    });
    ui.add_space(2.0);
}

fn badge(ui: &mut Ui, text: &str, color: Color32, _theme: &Theme) {
    let font = f_sb(9.0);
    let galley = ui.fonts(|f| f.layout_no_wrap(text.to_string(), font.clone(), color));
    let padding = egui::vec2(9.0, 3.0);
    let (rect, _) = ui.allocate_exact_size(galley.size() + padding * 2.0, Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        Rounding::same(999.0),
        Color32::from_rgba_premultiplied(255, 255, 255, 10),
    );
    painter.rect_stroke(
        rect,
        Rounding::same(999.0),
        Stroke::new(1.0, color.linear_multiply(0.6)),
    );
    painter.text(rect.center(), Align2::CENTER_CENTER, text, font, color);
}

/// Animated "LIVE" badge with a pulsing green dot.
fn live_badge(ui: &mut Ui, theme: &Theme) {
    let font = f_sb(9.0);
    let text = "LIVE";
    let text_galley = ui.fonts(|f| f.layout_no_wrap(text.to_string(), font.clone(), theme.teal));
    let dot_w = 12.0;
    let pad_x = 10.0;
    let pad_y = 3.0;
    let total = egui::vec2(
        dot_w + 4.0 + text_galley.size().x + pad_x * 2.0,
        text_galley.size().y + pad_y * 2.0,
    );
    let (rect, _) = ui.allocate_exact_size(total, Sense::hover());
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(60));
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        Rounding::same(999.0),
        Color32::from_rgba_premultiplied(255, 255, 255, 10),
    );
    painter.rect_stroke(
        rect,
        Rounding::same(999.0),
        Stroke::new(1.0, theme.teal.linear_multiply(0.6)),
    );

    // Pulsing dot
    let t = ui.input(|i| i.time) as f32;
    let phase = (t * 2.0).sin() * 0.5 + 0.5;
    let dot_center = Pos2::new(rect.min.x + pad_x + 4.0, rect.center().y);
    let ring_r = 5.0 + phase * 4.0;
    let ring_alpha = (170.0 * (1.0 - phase)) as u8;
    painter.circle_filled(
        dot_center,
        ring_r,
        Color32::from_rgba_unmultiplied(theme.teal.r(), theme.teal.g(), theme.teal.b(), ring_alpha),
    );
    painter.circle_filled(dot_center, 3.5, theme.teal);

    let text_pos = Pos2::new(dot_center.x + 4.0 + 4.0, rect.center().y);
    painter.text(text_pos, Align2::LEFT_CENTER, text, font, theme.teal);
}

// ─────────────────────────────────────────────────────────────
// Device list card
// ─────────────────────────────────────────────────────────────

fn device_list_card(
    ui: &mut Ui,
    devices: &[DeviceInfo],
    total: usize,
    state: &AppState,
    theme: &Theme,
    _target_h: f32,
    ui_state: &mut UiState,
) {
    egui::Frame::none()
        .fill(theme.card)
        .stroke(Stroke::new(1.0, theme.card_outline))
        .rounding(Rounding::same(theme.card_radius))
        .inner_margin(Margin::symmetric(24.0, 20.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("DEVICES")
                        .font(f_sb(10.0))
                        .color(theme.text_muted),
                );
                ui.label(
                    RichText::new(format!("({}/{})", devices.len(), total))
                        .font(f_reg(10.5))
                        .color(theme.text_dim),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let secs = state.config.lock().unwrap().refresh_interval_secs;
                    ui.label(
                        RichText::new(format!("↻ {secs}s"))
                            .font(f_mono(10.0))
                            .color(theme.text_dim),
                    );
                });
            });
            ui.add_space(14.0);

            if devices.is_empty() {
                empty_state(ui, theme);
                return;
            }

            let mut ci = 0usize;
            let chevron_tex = ui_state.icons.as_ref().map(|i| i.chevron.clone());

            for (i, d) in devices.iter().enumerate() {
                let accent = if d.connected {
                    let c = theme.accent_for(ci);
                    ci += 1;
                    c
                } else {
                    theme.text_dim
                };
                let open_detail = device_row(ui, d, accent, state, theme, chevron_tex.as_ref());
                if open_detail {
                    ui_state.selected_device = Some(d.address.clone());
                    ui_state.tab = Tab::Detail;
                }
                if i + 1 < devices.len() {
                    hairline(ui, theme);
                }
            }
        });
}

fn device_row(
    ui: &mut Ui,
    d: &DeviceInfo,
    accent: Color32,
    state: &AppState,
    theme: &Theme,
    chevron_tex: Option<&egui::TextureHandle>,
) -> bool {
    let mut open_detail = false;

    // Fixed row height. Every visible element pins to the row's single
    // centre-line, so alignment is deterministic instead of chasing egui's
    // nested-layout heuristics.
    const ROW_H: f32 = 60.0;
    const DOT_W: f32 = 16.0;
    const CHEVRON_W: f32 = 28.0;
    const ACTION_W: f32 = 100.0;
    const BAR_W: f32 = 36.0;
    const PCT_W: f32 = 34.0;
    const BATTERY_W: f32 = BAR_W + 8.0 + PCT_W; // bar + gap + text
    const GAP: f32 = 12.0;
    const BTN_H: f32 = 30.0;

    let full_w = ui.available_width();
    let (row_rect, _) = ui.allocate_exact_size(egui::vec2(full_w, ROW_H), Sense::hover());
    let cy = row_rect.center().y;

    // Column x-coordinates, laid out right-to-left so the info block gets
    // whatever remains on the left.
    let chevron_right = row_rect.right();
    let chevron_left = chevron_right - CHEVRON_W;
    let action_right = chevron_left - GAP;
    let action_left = action_right - ACTION_W;
    let battery_right = action_left - GAP;
    let battery_left = battery_right - BATTERY_W;
    let info_left = row_rect.left() + DOT_W + GAP;
    let info_right = battery_left - GAP;
    let info_w = (info_right - info_left).max(60.0);

    // ── Dot ────────────────────────────────────────────────────────────
    let dot_center = egui::pos2(row_rect.left() + DOT_W / 2.0, cy);
    let painter = ui.painter().clone();
    if d.connected {
        painter.circle_filled(dot_center, 6.0, accent);
    } else {
        painter.circle_stroke(dot_center, 5.5, Stroke::new(1.5, theme.text_dim));
    }

    // ── Info block: name (line 1) + address + tags (line 2) ────────────
    let name_color = if d.connected {
        theme.text
    } else {
        theme.text_muted
    };
    let name_text = format!("{} {}", device_emoji(d.icon.as_deref()), &d.name);
    let name_font = f_med(13.5);
    let addr_font = f_mono(10.0);
    let tag_font = f_sb(9.0);
    let line_gap = 4.0;

    let name_galley = ui.fonts(|f| f.layout(name_text, name_font.clone(), name_color, info_w));
    let addr_galley = ui.fonts(|f| {
        f.layout(
            d.address.clone(),
            addr_font.clone(),
            theme.text_dim,
            info_w,
        )
    });
    let stack_h = name_galley.size().y + line_gap + addr_galley.size().y;
    let name_top = cy - stack_h / 2.0;
    let addr_top = name_top + name_galley.size().y + line_gap;

    // Truncate name if it doesn't fit — draw the galley clipped
    painter.galley(egui::pos2(info_left, name_top), name_galley.clone(), name_color);

    // Address line — mono then optional TRUSTED / BLOCKED tags to its right
    let addr_w = addr_galley.size().x;
    painter.galley(
        egui::pos2(info_left, addr_top),
        addr_galley.clone(),
        theme.text_dim,
    );
    let mut tag_cursor = info_left + addr_w + 8.0;
    let addr_baseline = addr_top + addr_galley.size().y / 2.0;
    if d.trusted {
        let g = ui.fonts(|f| f.layout_no_wrap("· TRUSTED".into(), tag_font.clone(), theme.purple));
        painter.galley(
            egui::pos2(tag_cursor, addr_baseline - g.size().y / 2.0),
            g.clone(),
            theme.purple,
        );
        tag_cursor += g.size().x + 6.0;
    }
    if d.blocked {
        let g = ui.fonts(|f| f.layout_no_wrap("· BLOCKED".into(), tag_font.clone(), theme.red));
        painter.galley(
            egui::pos2(tag_cursor, addr_baseline - g.size().y / 2.0),
            g,
            theme.red,
        );
    }

    // ── Battery cell: [bar] gap "80%" — vertically centred on cy ───────
    match d.battery {
        Some(pct) => {
            let bcol = theme.battery_color(pct);
            let bar_h = 5.0;
            let bar_rect = Rect::from_min_size(
                egui::pos2(battery_left, cy - bar_h / 2.0),
                egui::vec2(BAR_W, bar_h),
            );
            painter.rect_filled(bar_rect, Rounding::same(999.0), theme.pill_track);
            let fill_w = BAR_W * (pct as f32 / 100.0).clamp(0.0, 1.0);
            let fill_rect = Rect::from_min_size(bar_rect.min, egui::vec2(fill_w, bar_h));
            painter.rect_filled(fill_rect, Rounding::same(999.0), bcol);
            painter.text(
                egui::pos2(battery_right, cy),
                Align2::RIGHT_CENTER,
                format!("{pct}%"),
                f_sb(13.0),
                bcol,
            );
        }
        None => {
            painter.text(
                egui::pos2(battery_right, cy),
                Align2::RIGHT_CENTER,
                "—",
                f_light(15.0),
                theme.text_dim,
            );
        }
    }

    // ── Action button ──────────────────────────────────────────────────
    let btn_rect = Rect::from_min_size(
        egui::pos2(action_left, cy - BTN_H / 2.0),
        egui::vec2(ACTION_W, BTN_H),
    );
    let (label, btn_color, cmd) = if d.connected {
        (
            "Disconnect",
            Some(theme.coral),
            Some(BluetoothCommand::Disconnect(d.address.clone())),
        )
    } else if d.paired {
        (
            "Connect",
            Some(theme.teal),
            Some(BluetoothCommand::Connect(d.address.clone())),
        )
    } else {
        ("Not paired", None, None)
    };
    if let (Some(bg), Some(cmd)) = (btn_color, cmd) {
        let btn_id = ui.id().with(("btn", d.address.as_str()));
        let btn_resp = ui.interact(btn_rect, btn_id, Sense::click());
        let actual_bg = if btn_resp.hovered() {
            Color32::from_rgba_unmultiplied(
                (bg.r() as f32 * 1.06).min(255.0) as u8,
                (bg.g() as f32 * 1.06).min(255.0) as u8,
                (bg.b() as f32 * 1.06).min(255.0) as u8,
                255,
            )
        } else {
            bg
        };
        painter.rect_filled(btn_rect, Rounding::same(theme.pill_radius), actual_bg);
        painter.text(
            btn_rect.center(),
            Align2::CENTER_CENTER,
            label,
            f_sb(11.5),
            theme.on_accent(),
        );
        if btn_resp.clicked() {
            let _ = state.cmd_tx.send(cmd);
        }
    } else {
        painter.text(
            btn_rect.center(),
            Align2::CENTER_CENTER,
            label,
            f_reg(11.0),
            theme.text_dim,
        );
    }

    // ── Chevron ────────────────────────────────────────────────────────
    let chev_rect = Rect::from_min_size(
        egui::pos2(chevron_left, cy - CHEVRON_W / 2.0),
        egui::vec2(CHEVRON_W, CHEVRON_W),
    );
    let chev_id = ui.id().with(("chev", d.address.as_str()));
    let chev_resp = ui.interact(chev_rect, chev_id, Sense::click());
    let chev_tint = if chev_resp.hovered() {
        theme.teal
    } else {
        theme.text_dim
    };
    if let Some(t) = chevron_tex {
        let icon_rect = Rect::from_center_size(chev_rect.center(), egui::vec2(14.0, 14.0));
        painter.image(
            t.id(),
            icon_rect,
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            chev_tint,
        );
    }
    if chev_resp.clicked() {
        open_detail = true;
    }

    // ── Row-wide click as an extra open-detail affordance + context menu.
    // Registered LAST so the button and chevron consume clicks in their area
    // first.
    let row_id = ui.id().with(("row", d.address.as_str()));
    let row_resp = ui.interact(row_rect, row_id, Sense::click());
    if row_resp.clicked() {
        open_detail = true;
    }

    row_resp.context_menu(|ui| {
        if ui.button("View details").clicked() {
            open_detail = true;
            ui.close_menu();
        }
        ui.separator();
        let trust_label = if d.trusted { "Untrust" } else { "Trust" };
        if ui.button(trust_label).clicked() {
            let _ = state
                .cmd_tx
                .send(BluetoothCommand::SetTrusted(d.address.clone(), !d.trusted));
            ui.close_menu();
        }
        let block_label = if d.blocked { "Unblock" } else { "Block" };
        if ui.button(block_label).clicked() {
            let _ = state
                .cmd_tx
                .send(BluetoothCommand::SetBlocked(d.address.clone(), !d.blocked));
            ui.close_menu();
        }
        ui.separator();
        if ui
            .button(RichText::new("Remove").color(theme.coral))
            .clicked()
        {
            let _ = state
                .cmd_tx
                .send(BluetoothCommand::Remove(d.address.clone()));
            ui.close_menu();
        }
    });

    open_detail
}

/// A small chevron-right button rendered in the row's rightmost slot.
fn chevron_button(
    ui: &mut Ui,
    tex: Option<&egui::TextureHandle>,
    theme: &Theme,
) -> egui::Response {
    let size = egui::vec2(28.0, 28.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    let bg = if resp.hovered() {
        theme.card_strong
    } else {
        Color32::TRANSPARENT
    };
    let tint = if resp.hovered() {
        theme.teal
    } else {
        theme.text_dim
    };
    if bg != Color32::TRANSPARENT {
        painter.rect_filled(rect, Rounding::same(theme.chip_radius.min(14.0)), bg);
    }
    if let Some(t) = tex {
        let icon_rect = Rect::from_center_size(rect.center(), egui::vec2(14.0, 14.0));
        painter.image(
            t.id(),
            icon_rect,
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            tint,
        );
    }
    resp
}

fn pill_button(ui: &mut Ui, label: &str, color: Color32, theme: &Theme) -> egui::Response {
    ui.add(
        egui::Button::new(
            RichText::new(label)
                .font(f_sb(11.5))
                .color(theme.on_accent()),
        )
        .fill(color)
        .rounding(Rounding::same(theme.pill_radius))
        .stroke(Stroke::NONE),
    )
}

fn battery_pill(ui: &mut Ui, pct: u8, color: Color32, theme: &Theme) {
    battery_pill_sized(ui, pct, color, theme, 50.0);
}

fn battery_pill_sized(ui: &mut Ui, pct: u8, color: Color32, theme: &Theme, width: f32) {
    let height = 5.0f32;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::same(999.0), theme.pill_track);
    let fill_w = width * (pct as f32 / 100.0).clamp(0.0, 1.0);
    let fill_rect = Rect::from_min_size(rect.min, egui::vec2(fill_w, height));
    painter.rect_filled(fill_rect, Rounding::same(999.0), color);
}

fn signal_bars_from_rssi(rssi: i16) -> u8 {
    if rssi > -50 {
        4
    } else if rssi > -65 {
        3
    } else if rssi > -80 {
        2
    } else {
        1
    }
}

fn signal_bars_widget(ui: &mut Ui, filled: u8, theme: &Theme) {
    let bar_w = 3.5;
    let gap = 2.0;
    let heights = [4.0, 7.0, 10.0, 13.0];
    let total_w = bar_w * 4.0 + gap * 3.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(total_w, 14.0), Sense::hover());
    let painter = ui.painter();
    let color = match filled {
        4 => theme.teal,
        3 => theme.yellow,
        2 => theme.orange,
        _ => theme.coral,
    };
    for i in 0..4 {
        let x = rect.min.x + i as f32 * (bar_w + gap);
        let y = rect.max.y - heights[i];
        let r = Rect::from_min_size(egui::pos2(x, y), egui::vec2(bar_w, heights[i]));
        let c = if (i as u8) < filled {
            color
        } else {
            theme.pill_track
        };
        painter.rect_filled(r, Rounding::same(1.5), c);
    }
}

fn hairline(ui: &mut Ui, theme: &Theme) {
    let h = 1.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), h), Sense::hover());
    let col = Color32::from_rgba_unmultiplied(
        theme.card_outline.r(),
        theme.card_outline.g(),
        theme.card_outline.b(),
        45,
    );
    ui.painter().rect_filled(rect, Rounding::ZERO, col);
}

fn empty_state(ui: &mut Ui, theme: &Theme) {
    ui.vertical_centered(|ui| {
        ui.add_space(30.0);
        ui.label(RichText::new("🎧").size(46.0));
        ui.add_space(6.0);
        ui.label(
            RichText::new("Nothing to show here")
                .font(f_med(14.0))
                .color(theme.text_muted),
        );
        ui.label(
            RichText::new("Try another tab, or pair a device")
                .font(f_reg(11.0))
                .color(theme.text_dim),
        );
        ui.add_space(20.0);
    });
}

fn device_emoji(icon: Option<&str>) -> &'static str {
    let icon = icon.unwrap_or("").to_lowercase();
    if icon.contains("audio") || icon.contains("headset") || icon.contains("headphone") {
        "🎧"
    } else if icon.contains("mouse") {
        "🖱"
    } else if icon.contains("keyboard") {
        "⌨"
    } else if icon.contains("phone") {
        "📱"
    } else if icon.contains("computer") {
        "💻"
    } else if icon.contains("gamepad") || icon.contains("joystick") {
        "🎮"
    } else if icon.contains("watch") {
        "⌚"
    } else if icon.contains("printer") {
        "🖨"
    } else if icon.contains("speaker") {
        "🔊"
    } else {
        "🔵"
    }
}

// ─────────────────────────────────────────────────────────────
// Discover
// ─────────────────────────────────────────────────────────────

fn discover(ui: &mut Ui, state: &AppState, ui_state: &mut UiState) {
    let theme = ui_state.theme;
    let scanning = state.scanning.load(Ordering::Relaxed);
    let elapsed = state
        .scan_started
        .lock()
        .unwrap()
        .map(|t| t.elapsed().as_secs_f32())
        .unwrap_or(0.0);
    let nearby = state.nearby.lock().unwrap().clone();

    let avail_h = ui.available_height().max(360.0);

    ui.columns(2, |cols| {
        cols[0].with_layout(Layout::top_down(Align::Center), |ui| {
            radar_card(ui, &theme, scanning, elapsed, nearby.len(), state, avail_h);
        });
        cols[1].with_layout(Layout::top_down(Align::Min), |ui| {
            nearby_list_card(ui, &nearby, &theme, state, avail_h);
        });
    });
}

fn radar_card(
    ui: &mut Ui,
    theme: &Theme,
    scanning: bool,
    elapsed: f32,
    n_found: usize,
    state: &AppState,
    _target_h: f32,
) {
    egui::Frame::none()
        .fill(theme.card)
        .stroke(Stroke::new(1.0, theme.card_outline))
        .rounding(Rounding::same(theme.card_radius))
        .inner_margin(Margin::symmetric(22.0, 22.0))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("RADAR")
                            .font(f_sb(10.0))
                            .color(theme.text_muted),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if scanning {
                            live_badge(ui, theme);
                        } else {
                            let font = f_sb(9.0);
                            let text = "IDLE";
                            let galley = ui.fonts(|f| {
                                f.layout_no_wrap(text.into(), font.clone(), theme.text_dim)
                            });
                            let pad = egui::vec2(9.0, 3.0);
                            let (rect, _) =
                                ui.allocate_exact_size(galley.size() + pad * 2.0, Sense::hover());
                            ui.painter().rect_stroke(
                                rect,
                                Rounding::same(999.0),
                                Stroke::new(1.0, theme.card_outline),
                            );
                            ui.painter().text(
                                rect.center(),
                                Align2::CENTER_CENTER,
                                text,
                                font,
                                theme.text_dim,
                            );
                        }
                    });
                });
                ui.add_space(14.0);

                let size = 240.0;
                let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
                paint_radar(ui, rect, theme, scanning, elapsed);

                ui.add_space(14.0);
                ui.label(
                    RichText::new(format!(
                        "{n_found} DEVICE{}",
                        if n_found == 1 { "" } else { "S" }
                    ))
                    .font(f_sb(11.0))
                    .color(theme.text),
                );
                ui.label(
                    RichText::new(if scanning {
                        format!("Elapsed {:.0}s", elapsed)
                    } else {
                        "Not scanning".into()
                    })
                    .font(f_reg(10.5))
                    .color(theme.text_dim),
                );

                ui.add_space(16.0);
                let (label, color, cmd) = if scanning {
                    ("Stop scan", theme.coral, BluetoothCommand::StopDiscovery)
                } else {
                    ("Start scan", theme.teal, BluetoothCommand::StartDiscovery)
                };
                let btn = egui::Button::new(
                    RichText::new(label)
                        .font(f_sb(12.0))
                        .color(theme.on_accent()),
                )
                .fill(color)
                .rounding(Rounding::same(999.0))
                .stroke(Stroke::NONE);
                if ui.add_sized([160.0, 34.0], btn).clicked() {
                    let _ = state.cmd_tx.send(cmd);
                }
            });
        });
}

fn paint_radar(ui: &Ui, rect: Rect, theme: &Theme, scanning: bool, elapsed: f32) {
    let painter = ui.painter();
    let center = rect.center();
    let r_out = rect.width().min(rect.height()) / 2.0 - 4.0;

    // Concentric rings (dashed)
    for i in 1..=4 {
        let r = r_out * (i as f32 / 4.0);
        painter.circle_stroke(center, r, Stroke::new(1.0, theme.card_outline));
    }
    // Cross axes
    painter.line_segment(
        [
            Pos2::new(rect.min.x + 8.0, center.y),
            Pos2::new(rect.max.x - 8.0, center.y),
        ],
        Stroke::new(1.0, theme.card_outline.linear_multiply(0.6)),
    );
    painter.line_segment(
        [
            Pos2::new(center.x, rect.min.y + 8.0),
            Pos2::new(center.x, rect.max.y - 8.0),
        ],
        Stroke::new(1.0, theme.card_outline.linear_multiply(0.6)),
    );

    if scanning {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(40));

        // Sweep line
        let angle = -std::f32::consts::FRAC_PI_2 + elapsed * 2.4;
        let sweep_end = center + egui::vec2(angle.cos(), angle.sin()) * r_out;
        painter.line_segment([center, sweep_end], Stroke::new(2.5, theme.teal));

        // Trailing gradient wedge
        let seg = 24;
        let mut pts = vec![center];
        for i in 0..=seg {
            let t = i as f32 / seg as f32;
            let a = angle - std::f32::consts::PI * 0.35 * t;
            pts.push(center + egui::vec2(a.cos(), a.sin()) * r_out);
        }
        let wedge_color =
            Color32::from_rgba_unmultiplied(theme.teal.r(), theme.teal.g(), theme.teal.b(), 40);
        painter.add(Shape::convex_polygon(pts, wedge_color, Stroke::NONE));

        // Pulse rings from center
        for phase_offset in [0.0f32, 0.5, 1.0] {
            let phase = ((elapsed * 0.7 + phase_offset) % 1.5) / 1.5;
            let ring_r = 6.0 + phase * (r_out - 12.0);
            let alpha = ((1.0 - phase) * 90.0) as u8;
            painter.circle_stroke(
                center,
                ring_r,
                Stroke::new(
                    1.2,
                    Color32::from_rgba_unmultiplied(
                        theme.teal.r(),
                        theme.teal.g(),
                        theme.teal.b(),
                        alpha,
                    ),
                ),
            );
        }
    }

    // Static center dot (drawn last so it's always on top)
    painter.circle_filled(center, 10.0, theme.card);
    painter.circle_filled(
        center,
        8.0,
        Color32::from_rgba_unmultiplied(theme.teal.r(), theme.teal.g(), theme.teal.b(), 60),
    );
    painter.circle_filled(center, 4.5, theme.teal);
}

fn nearby_list_card(
    ui: &mut Ui,
    nearby: &[DiscoveredDevice],
    theme: &Theme,
    state: &AppState,
    _target_h: f32,
) {
    egui::Frame::none()
        .fill(theme.card)
        .stroke(Stroke::new(1.0, theme.card_outline))
        .rounding(Rounding::same(theme.card_radius))
        .inner_margin(Margin::symmetric(24.0, 20.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("NEARBY")
                        .font(f_sb(10.0))
                        .color(theme.text_muted),
                );
                ui.label(
                    RichText::new(format!("({})", nearby.len()))
                        .font(f_reg(10.5))
                        .color(theme.text_dim),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new("RSSI ↓")
                            .font(f_mono(10.0))
                            .color(theme.text_dim),
                    );
                });
            });
            ui.add_space(14.0);

            if nearby.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(RichText::new("📡").size(46.0));
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("Nothing nearby yet")
                            .font(f_med(14.0))
                            .color(theme.text_muted),
                    );
                    ui.label(
                        RichText::new("Start a scan and put your device in pairing mode")
                            .font(f_reg(11.0))
                            .color(theme.text_dim),
                    );
                });
            } else {
                for (i, d) in nearby.iter().enumerate() {
                    nearby_row(ui, d, theme, state);
                    if i + 1 < nearby.len() {
                        hairline(ui, theme);
                    }
                }
            }

            // Tip card — always visible at the bottom of the right column
            ui.add_space(12.0);
            tip_card(ui, theme);
        });
}

fn tip_card(ui: &mut Ui, theme: &Theme) {
    let tint = Color32::from_rgba_unmultiplied(theme.teal.r(), theme.teal.g(), theme.teal.b(), 22);
    let stroke = Color32::from_rgba_unmultiplied(theme.teal.r(), theme.teal.g(), theme.teal.b(), 90);
    egui::Frame::none()
        .fill(tint)
        .stroke(Stroke::new(1.0, stroke))
        .rounding(Rounding::same(theme.card_radius.min(14.0)))
        .inner_margin(Margin::symmetric(14.0, 12.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new("A QUIET NOTE")
                    .font(f_sb(10.0))
                    .color(theme.teal),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new(
                    "Not seeing your device? Put it in pairing mode first — \
                     usually a long press on its pair button until the LED blinks.",
                )
                .font(f_reg(11.5))
                .color(theme.text_muted),
            );
        });
}

fn nearby_row(ui: &mut Ui, d: &DiscoveredDevice, theme: &Theme, state: &AppState) {
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), 52.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            let bars = d.rssi.map(signal_bars_from_rssi).unwrap_or(1);
            let marker_color = match bars {
                4 => theme.teal,
                3 => theme.yellow,
                2 => theme.orange,
                _ => theme.coral,
            };
            let (marker, _) = ui.allocate_exact_size(egui::vec2(4.0, 32.0), Sense::hover());
            ui.painter()
                .rect_filled(marker, Rounding::same(2.0), marker_color);
            ui.add_space(10.0);

            ui.vertical(|ui| {
                let name = d.name.clone().unwrap_or_else(|| "Unknown device".into());
                ui.label(RichText::new(name).font(f_med(13.0)).color(theme.text));
                ui.label(
                    RichText::new(&d.address)
                        .font(f_mono(10.0))
                        .color(theme.text_dim),
                );
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let btn = egui::Button::new(
                    RichText::new("Pair")
                        .font(f_sb(11.0))
                        .color(theme.on_accent()),
                )
                .fill(theme.teal)
                .rounding(Rounding::same(999.0))
                .stroke(Stroke::NONE);
                if ui.add(btn).clicked() {
                    let _ = state.cmd_tx.send(BluetoothCommand::Pair(d.address.clone()));
                }
                ui.add_space(10.0);
                if let Some(r) = d.rssi {
                    ui.label(
                        RichText::new(format!("{r}"))
                            .font(f_sb(14.0))
                            .color(marker_color),
                    );
                    ui.label(RichText::new("dBm").font(f_reg(9.5)).color(theme.text_dim));
                }
            });
        },
    );
}

// ─────────────────────────────────────────────────────────────
// Detail
// ─────────────────────────────────────────────────────────────

fn detail(ui: &mut Ui, state: &AppState, ui_state: &mut UiState) {
    let theme = ui_state.theme;
    let addr = match ui_state.selected_device.as_ref() {
        Some(a) => a.clone(),
        None => {
            ui.vertical_centered(|ui| {
                ui.add_space(60.0);
                ui.label(
                    RichText::new("Select a device from the dashboard")
                        .font(f_reg(13.0))
                        .color(theme.text_muted),
                );
            });
            return;
        }
    };
    let devices = state.devices.lock().unwrap().clone();
    let device = match devices.iter().find(|d| d.address == addr) {
        Some(d) => d.clone(),
        None => {
            ui.vertical_centered(|ui| {
                ui.add_space(60.0);
                ui.label(
                    RichText::new("Device no longer available")
                        .font(f_reg(13.0))
                        .color(theme.text_muted),
                );
                if ui.button("Back to dashboard").clicked() {
                    ui_state.tab = Tab::Dashboard;
                }
            });
            return;
        }
    };

    detail_hero(ui, &device, &theme);
    ui.add_space(14.0);
    if let Some(components) = device.components.as_ref() {
        detail_components(ui, components, &theme);
        ui.add_space(14.0);
    }
    detail_metrics(ui, &device, &theme);
    ui.add_space(14.0);
    detail_signal_chart(ui, &device, &theme, state);
    ui.add_space(14.0);
    detail_info(ui, &device, &theme);
    ui.add_space(14.0);
    detail_actions(ui, &device, &theme, state);

    // MMA / Xiaomi SPP probe panel — only surfaced when the device
    // advertises the Xiaomi SPP UUID, so it stays hidden for
    // AirPods / Sony / etc.
    if device
        .uuids
        .iter()
        .any(|u| u.eq_ignore_ascii_case(crate::xiaomi_mma::XIAOMI_SPP_UUID))
    {
        ui.add_space(14.0);
        detail_mma_probe(ui, &device, &theme, state);
    }

    if !device.uuids.is_empty() {
        ui.add_space(14.0);
        detail_services(ui, &device, &theme);
    }
    ui.add_space(24.0);
}

fn detail_mma_probe(ui: &mut Ui, d: &DeviceInfo, theme: &Theme, state: &AppState) {
    egui::Frame::none()
        .fill(theme.card)
        .stroke(Stroke::new(1.0, theme.card_outline))
        .rounding(Rounding::same(theme.card_radius.min(16.0)))
        .inner_margin(Margin::symmetric(20.0, 14.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("MMA PROBE · XIAOMI SPP")
                        .font(f_sb(10.0))
                        .color(theme.text_dim),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let btn = egui::Button::new(
                        RichText::new("Probe now")
                            .font(f_sb(11.0))
                            .color(theme.on_accent()),
                    )
                    .fill(theme.teal)
                    .rounding(Rounding::same(theme.pill_radius))
                    .stroke(Stroke::NONE);
                    if ui.add(btn).clicked() {
                        let _ = state
                            .cmd_tx
                            .send(BluetoothCommand::ProbeMma(d.address.clone()));
                    }
                });
            });
            ui.add_space(8.0);
            let log = state
                .mma_log
                .lock()
                .unwrap()
                .get(&d.address)
                .cloned()
                .unwrap_or_else(|| {
                    "Click 'Probe now' to open an RFCOMM socket, send a \
                     candidate MMA frame, and hex-dump whatever the earbud \
                     writes back. Use this to iterate on opcodes."
                        .into()
                });
            ui.label(
                RichText::new(log)
                    .font(f_mono(10.5))
                    .color(theme.text_muted),
            );
        });
}

/// Line chart of the last ~3 minutes of RSSI samples for this device.
fn detail_signal_chart(ui: &mut Ui, d: &DeviceInfo, theme: &Theme, state: &AppState) {
    let samples: Vec<i16> = {
        let hist = state.rssi_history.lock().unwrap();
        hist.get(&d.address).map(|q| q.iter().copied().collect()).unwrap_or_default()
    };

    egui::Frame::none()
        .fill(theme.card)
        .stroke(Stroke::new(1.0, theme.card_outline))
        .rounding(Rounding::same(theme.card_radius.min(16.0)))
        .inner_margin(Margin::symmetric(20.0, 14.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new("SIGNAL — LAST 60 SAMPLES").font(f_sb(10.0)).color(theme.text_dim));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if let Some(last) = samples.last() {
                        ui.label(
                            RichText::new(format!("{last} dBm"))
                                .font(f_sb(11.0))
                                .color(theme.rssi_color(*last)),
                        );
                    } else {
                        ui.label(RichText::new("no readings yet").font(f_reg(10.5)).color(theme.text_dim));
                    }
                });
            });
            ui.add_space(8.0);

            let chart_h = 70.0;
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), chart_h),
                Sense::hover(),
            );
            paint_sparkline(ui, rect, &samples, theme);
        });
}

fn paint_sparkline(ui: &Ui, rect: Rect, samples: &[i16], theme: &Theme) {
    let painter = ui.painter();

    // Baseline grid (dashed-ish horizontal lines at -50 / -70 / -90)
    let grid = theme.card_outline;
    let range_top = -30.0f32;
    let range_bot = -100.0f32;
    for guide_dbm in [-50.0f32, -70.0f32, -90.0f32] {
        let t = (guide_dbm - range_top) / (range_bot - range_top);
        let y = rect.min.y + rect.height() * t;
        painter.line_segment(
            [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(grid.r(), grid.g(), grid.b(), 60)),
        );
        painter.text(
            egui::pos2(rect.min.x + 4.0, y - 8.0),
            Align2::LEFT_TOP,
            format!("{}", guide_dbm as i16),
            f_mono(9.0),
            theme.text_dim,
        );
    }

    if samples.len() < 2 {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "collecting samples…",
            f_reg(10.5),
            theme.text_dim,
        );
        return;
    }

    // Map each sample to a point in the chart
    let n = samples.len();
    let x_step = rect.width() / (n as f32 - 1.0).max(1.0);
    let map_y = |v: i16| -> f32 {
        let clamped = (v as f32).clamp(range_bot, range_top);
        let t = (clamped - range_top) / (range_bot - range_top);
        rect.min.y + rect.height() * t
    };

    let line_color = theme.rssi_color(*samples.last().unwrap());
    let pts: Vec<Pos2> = samples
        .iter()
        .enumerate()
        .map(|(i, v)| Pos2::new(rect.min.x + i as f32 * x_step, map_y(*v)))
        .collect();

    // Filled area under the line
    let mut fill_pts = pts.clone();
    fill_pts.push(Pos2::new(rect.max.x, rect.max.y));
    fill_pts.push(Pos2::new(rect.min.x, rect.max.y));
    let fill_color = Color32::from_rgba_unmultiplied(line_color.r(), line_color.g(), line_color.b(), 30);
    painter.add(Shape::convex_polygon(fill_pts, fill_color, Stroke::NONE));

    // The line itself
    painter.add(Shape::line(pts.clone(), Stroke::new(1.6, line_color)));

    // Last-point dot
    if let Some(&last_pt) = pts.last() {
        painter.circle_filled(last_pt, 3.0, line_color);
        painter.circle_stroke(last_pt, 5.0, Stroke::new(1.0, line_color.linear_multiply(0.5)));
    }
}

fn detail_hero(ui: &mut Ui, d: &DeviceInfo, theme: &Theme) {
    egui::Frame::none()
        .fill(theme.card)
        .stroke(Stroke::new(1.0, theme.card_outline))
        .rounding(Rounding::same(theme.card_radius.min(20.0)))
        .inner_margin(Margin::symmetric(22.0, 18.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                // Avatar tile
                let (avatar, _) = ui.allocate_exact_size(egui::vec2(56.0, 56.0), Sense::hover());
                let painter = ui.painter();
                painter.rect_filled(
                    avatar,
                    Rounding::same(14.0),
                    Color32::from_rgba_unmultiplied(
                        theme.teal.r(),
                        theme.teal.g(),
                        theme.teal.b(),
                        30,
                    ),
                );
                painter.text(
                    avatar.center(),
                    Align2::CENTER_CENTER,
                    device_emoji(d.icon.as_deref()),
                    FontId::proportional(28.0),
                    theme.text,
                );

                ui.add_space(14.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(&d.name).font(f_bold(18.0)).color(theme.text));
                    ui.label(
                        RichText::new(&d.address)
                            .font(f_mono(11.0))
                            .color(theme.text_dim),
                    );
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        if d.connected {
                            pill_status(ui, "● CONNECTED", theme.teal);
                        } else {
                            pill_status(ui, "○ DISCONNECTED", theme.text_dim);
                        }
                        if d.paired {
                            pill_status(ui, "PAIRED", theme.purple);
                        }
                        if d.trusted {
                            pill_status(ui, "TRUSTED", theme.yellow);
                        }
                        if d.blocked {
                            pill_status(ui, "BLOCKED", theme.red);
                        }
                    });
                });
            });
        });
}

fn pill_status(ui: &mut Ui, text: &str, color: Color32) {
    let font = f_sb(9.5);
    let galley = ui.fonts(|f| f.layout_no_wrap(text.into(), font.clone(), color));
    let pad = egui::vec2(9.0, 3.0);
    let (rect, _) = ui.allocate_exact_size(galley.size() + pad * 2.0, Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        Rounding::same(999.0),
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 30),
    );
    painter.rect_stroke(
        rect,
        Rounding::same(999.0),
        Stroke::new(1.0, color.linear_multiply(0.7)),
    );
    painter.text(rect.center(), Align2::CENTER_CENTER, text, font, color);
    ui.add_space(4.0);
}

/// Renders the "COMPONENTS" card for TWS earbuds: three columns — L bud,
/// case, R bud — each with a big battery %, a colored bar, and a charging
/// indicator when known.
fn detail_components(ui: &mut Ui, c: &crate::vendors::Components, theme: &Theme) {
    egui::Frame::none()
        .fill(theme.card)
        .stroke(Stroke::new(1.0, theme.card_outline))
        .rounding(Rounding::same(theme.card_radius.min(16.0)))
        .inner_margin(Margin::symmetric(20.0, 16.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("COMPONENTS")
                        .font(f_sb(10.0))
                        .color(theme.text_dim),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(c.source)
                            .font(f_mono(10.5))
                            .color(theme.text_muted),
                    );
                });
            });
            ui.add_space(12.0);

            ui.columns(3, |cols| {
                component_cell(&mut cols[0], "LEFT BUD", "🎧", c.left_battery, c.left_charging, theme);
                component_cell(&mut cols[1], "CASE", "🔋", c.case_battery, c.case_charging, theme);
                component_cell(&mut cols[2], "RIGHT BUD", "🎧", c.right_battery, c.right_charging, theme);
            });

            // Small footer with in-ear / lid states when known
            let mut chips: Vec<(String, Color32)> = Vec::new();
            if let Some(l) = c.in_ear_left {
                chips.push((
                    format!("L · {}", if l { "in ear" } else { "out" }),
                    if l { theme.teal } else { theme.text_dim },
                ));
            }
            if let Some(r) = c.in_ear_right {
                chips.push((
                    format!("R · {}", if r { "in ear" } else { "out" }),
                    if r { theme.teal } else { theme.text_dim },
                ));
            }
            if let Some(open) = c.case_open {
                chips.push((
                    format!("case {}", if open { "open" } else { "closed" }),
                    if open { theme.yellow } else { theme.text_dim },
                ));
            }
            if !chips.is_empty() {
                ui.add_space(10.0);
                ui.horizontal_wrapped(|ui| {
                    for (text, color) in chips {
                        component_chip(ui, &text, color, theme);
                    }
                });
            }
        });
}

fn component_cell(
    ui: &mut Ui,
    label: &str,
    emoji: &str,
    battery: Option<u8>,
    charging: Option<bool>,
    theme: &Theme,
) {
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(label)
                .font(f_sb(9.5))
                .color(theme.text_dim),
        );
        ui.add_space(6.0);
        ui.label(RichText::new(emoji).size(22.0));
        ui.add_space(4.0);
        match battery {
            Some(pct) => {
                let color = theme.battery_color(pct);
                ui.label(
                    RichText::new(format!("{pct}%"))
                        .font(hero_font(theme, 24.0))
                        .color(color),
                );
                ui.add_space(6.0);
                // Small horizontal bar
                let width = 90.0f32;
                let height = 5.0f32;
                let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), Sense::hover());
                let painter = ui.painter();
                painter.rect_filled(rect, Rounding::same(999.0), theme.pill_track);
                let fill_w = width * (pct as f32 / 100.0).clamp(0.0, 1.0);
                painter.rect_filled(
                    Rect::from_min_size(rect.min, egui::vec2(fill_w, height)),
                    Rounding::same(999.0),
                    color,
                );
                if let Some(true) = charging {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("⚡ charging")
                            .font(f_sb(9.5))
                            .color(theme.yellow),
                    );
                }
            }
            None => {
                ui.label(
                    RichText::new("—")
                        .font(hero_font(theme, 24.0))
                        .color(theme.text_dim),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new("not reporting")
                        .font(f_reg(9.5))
                        .color(theme.text_dim),
                );
            }
        }
    });
}

fn component_chip(ui: &mut Ui, text: &str, color: Color32, theme: &Theme) {
    let font = f_sb(9.5);
    let galley = ui.fonts(|f| f.layout_no_wrap(text.to_string(), font.clone(), color));
    let padding = egui::vec2(9.0, 3.0);
    let (rect, _) = ui.allocate_exact_size(galley.size() + padding * 2.0, Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        Rounding::same(theme.pill_radius),
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 22),
    );
    painter.rect_stroke(
        rect,
        Rounding::same(theme.pill_radius),
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 90)),
    );
    painter.text(rect.center(), Align2::CENTER_CENTER, text, font, color);
    ui.add_space(4.0);
}

fn detail_metrics(ui: &mut Ui, d: &DeviceInfo, theme: &Theme) {
    ui.columns(2, |cols| {
        cols[0].vertical(|ui| {
            metric_cell(
                ui,
                theme,
                "BATTERY",
                d.battery
                    .map(|b| format!("{b}"))
                    .unwrap_or_else(|| "—".into()),
                "%",
                d.battery
                    .map(|b| theme.battery_color(b))
                    .unwrap_or(theme.text_dim),
            );
        });
        cols[1].vertical(|ui| {
            metric_cell(
                ui,
                theme,
                "SIGNAL",
                d.rssi.map(|r| format!("{r}")).unwrap_or_else(|| "—".into()),
                "dBm",
                d.rssi
                    .map(|r| theme.rssi_color(r))
                    .unwrap_or(theme.text_dim),
            );
        });
    });
}

fn metric_cell(ui: &mut Ui, theme: &Theme, label: &str, value: String, unit: &str, color: Color32) {
    egui::Frame::none()
        .fill(theme.card)
        .stroke(Stroke::new(1.0, theme.card_outline))
        .rounding(Rounding::same(16.0))
        .inner_margin(Margin::symmetric(18.0, 14.0))
        .show(ui, |ui| {
            ui.label(RichText::new(label).font(f_sb(10.0)).color(theme.text_dim));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(value).font(f_light(38.0)).color(color));
                ui.label(RichText::new(unit).font(f_reg(12.0)).color(theme.text_dim));
            });
        });
}

fn detail_info(ui: &mut Ui, d: &DeviceInfo, theme: &Theme) {
    egui::Frame::none()
        .fill(theme.card)
        .stroke(Stroke::new(1.0, theme.card_outline))
        .rounding(Rounding::same(theme.card_radius.min(16.0)))
        .inner_margin(Margin::symmetric(20.0, 14.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                RichText::new("PROPERTIES")
                    .font(f_sb(10.0))
                    .color(theme.text_dim),
            );
            ui.add_space(8.0);
            kv_line(
                ui,
                theme,
                "Type",
                &d.icon.clone().unwrap_or_else(|| "—".into()),
            );
            kv_line(
                ui,
                theme,
                "Vendor",
                &d.manufacturer.clone().unwrap_or_else(|| "—".into()),
            );
            kv_line(
                ui,
                theme,
                "Class",
                &d.class_of_device
                    .map(|c| format!("0x{:06X}", c))
                    .unwrap_or_else(|| "—".into()),
            );
            kv_line(
                ui,
                theme,
                "Tx power",
                &d.tx_power
                    .map(|t| format!("{t} dBm"))
                    .unwrap_or_else(|| "—".into()),
            );
            kv_line(ui, theme, "Services", &format!("{} UUIDs", d.uuids.len()));
        });
}

fn kv_line(ui: &mut Ui, theme: &Theme, k: &str, v: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(k).font(f_reg(11.5)).color(theme.text_muted));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(v).font(f_mono(11.0)).color(theme.text));
        });
    });
    ui.add_space(2.0);
    hairline(ui, theme);
    ui.add_space(2.0);
}

fn detail_actions(ui: &mut Ui, d: &DeviceInfo, theme: &Theme, state: &AppState) {
    ui.horizontal(|ui| {
        let (label, color, cmd) = if d.connected {
            (
                "Disconnect",
                theme.coral,
                BluetoothCommand::Disconnect(d.address.clone()),
            )
        } else if d.paired {
            (
                "Connect",
                theme.teal,
                BluetoothCommand::Connect(d.address.clone()),
            )
        } else {
            (
                "Pair",
                theme.teal,
                BluetoothCommand::Pair(d.address.clone()),
            )
        };
        let btn = egui::Button::new(
            RichText::new(label)
                .font(f_sb(12.0))
                .color(theme.on_accent()),
        )
        .fill(color)
        .rounding(Rounding::same(999.0))
        .stroke(Stroke::NONE);
        if ui.add_sized([130.0, 34.0], btn).clicked() {
            let _ = state.cmd_tx.send(cmd);
        }
        ui.add_space(6.0);

        let trust_label = if d.trusted { "Untrust" } else { "Trust" };
        if ghost_btn(ui, trust_label, theme).clicked() {
            let _ = state
                .cmd_tx
                .send(BluetoothCommand::SetTrusted(d.address.clone(), !d.trusted));
        }
        ui.add_space(6.0);

        let block_label = if d.blocked { "Unblock" } else { "Block" };
        if ghost_btn(ui, block_label, theme).clicked() {
            let _ = state
                .cmd_tx
                .send(BluetoothCommand::SetBlocked(d.address.clone(), !d.blocked));
        }
        ui.add_space(6.0);

        if ghost_btn(ui, "Remove", theme).clicked() {
            let _ = state
                .cmd_tx
                .send(BluetoothCommand::Remove(d.address.clone()));
        }
    });
}

fn ghost_btn(ui: &mut Ui, label: &str, theme: &Theme) -> egui::Response {
    let btn = egui::Button::new(
        RichText::new(label)
            .font(f_sb(11.5))
            .color(theme.text_muted),
    )
    .fill(theme.card)
    .rounding(Rounding::same(999.0))
    .stroke(Stroke::new(1.0, theme.card_outline));
    ui.add_sized([100.0, 34.0], btn)
}

fn detail_services(ui: &mut Ui, d: &DeviceInfo, theme: &Theme) {
    egui::Frame::none()
        .fill(theme.card)
        .stroke(Stroke::new(1.0, theme.card_outline))
        .rounding(Rounding::same(theme.card_radius.min(16.0)))
        .inner_margin(Margin::symmetric(20.0, 14.0))
        .show(ui, |ui| {
            // Force the card to span the whole column, otherwise the narrow
            // monospace UUID strings would let it collapse into a thin box.
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("SERVICES")
                        .font(f_sb(10.0))
                        .color(theme.text_dim),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} UUIDs", d.uuids.len()))
                            .font(f_reg(10.5))
                            .color(theme.text_dim),
                    );
                });
            });
            ui.add_space(8.0);
            for u in &d.uuids {
                ui.label(
                    RichText::new(u)
                        .font(f_mono(10.5))
                        .color(theme.text_muted),
                );
            }
        });
}

// ─────────────────────────────────────────────────────────────
// Settings
// ─────────────────────────────────────────────────────────────

fn settings(ui: &mut Ui, state: &AppState, ui_state: &mut UiState) {
    let theme = ui_state.theme;
    // The body-level ScrollArea in `render()` handles overflow for every tab.
    settings_group(ui, "APPEARANCE", &theme, |ui| {
        theme_picker(ui, ui_state, state);
    });
    ui.add_space(18.0);

    settings_group(ui, "PREFERENCES", &theme, |ui| {
        let mut cfg = state.config.lock().unwrap().clone();
        let mut changed = false;
        changed |= toggle_row(
            ui,
            "🔋",
            "Low battery alert",
            "Notify when a connected device drops below the threshold",
            &mut cfg.low_battery_alert,
            &theme,
        );
        changed |= toggle_row(
            ui,
            "🖼",
            "Close to tray",
            "Keep the app running in the background when the window is closed",
            &mut cfg.close_to_tray,
            &theme,
        );
        let before_autostart = cfg.autostart;
        let autostart_changed = toggle_row(
            ui,
            "🚀",
            "Launch at login",
            "Start with the system session",
            &mut cfg.autostart,
            &theme,
        );
        changed |= autostart_changed;
        changed |= interval_row(
            ui,
            "⏱",
            "Refresh interval",
            "How often devices are polled",
            &mut cfg.refresh_interval_secs,
            &theme,
        );
        if changed {
            *state.config.lock().unwrap() = cfg.clone();
            cfg.save();
            if autostart_changed && cfg.autostart != before_autostart {
                if let Err(e) = crate::autostart::set(cfg.autostart) {
                    *state.last_error.lock().unwrap() = Some(format!("Autostart: {e}"));
                }
            }
        }
    });
    ui.add_space(18.0);

    settings_group(ui, "ADAPTER", &theme, |ui| {
        let powered = state.adapter_powered.load(Ordering::Relaxed);
        let adapter_name = state
            .adapter_name
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| "hci0".into());
        info_row(ui, "📡", "Controller", &adapter_name, &theme);
        info_row(
            ui,
            "⚡",
            "Powered",
            if powered { "yes" } else { "no" },
            &theme,
        );
    });
    ui.add_space(18.0);

    settings_group(ui, "ABOUT", &theme, |ui| {
        info_row(
            ui,
            "🔵",
            "Bluetooth Monitor",
            env!("CARGO_PKG_VERSION"),
            &theme,
        );
        info_row(ui, "🛠", "Engine", "eframe · egui", &theme);
        info_row(ui, "📚", "Stack", "bluer · tokio · ksni", &theme);
    });
    ui.add_space(24.0);
}

fn settings_group(ui: &mut Ui, title: &str, theme: &Theme, add: impl FnOnce(&mut Ui)) {
    ui.label(
        RichText::new(title)
            .font(f_sb(10.5))
            .color(theme.text_muted),
    );
    ui.add_space(8.0);
    egui::Frame::none()
        .fill(theme.card)
        .stroke(Stroke::new(1.0, theme.card_outline))
        .rounding(Rounding::same(16.0))
        .inner_margin(Margin::symmetric(18.0, 14.0))
        .show(ui, add);
}

fn theme_picker(ui: &mut Ui, ui_state: &mut UiState, state: &AppState) {
    let current = ui_state.theme.kind;
    let theme = ui_state.theme;

    ui.label(RichText::new("Theme").font(f_sb(13.5)).color(theme.text));
    ui.label(
        RichText::new("Choose how the app looks. Applies instantly.")
            .font(f_reg(11.0))
            .color(theme.text_dim),
    );
    ui.add_space(12.0);

    ui.horizontal(|ui| {
        for kind in ThemeKind::ALL {
            let selected = kind == current;
            let clicked = theme_swatch(ui, kind, selected);
            if clicked {
                let new_theme = Theme::from_kind(kind);
                ui_state.theme = new_theme;
                theme::apply_style(ui.ctx(), &new_theme);
                let mut cfg = state.config.lock().unwrap().clone();
                cfg.theme = kind;
                *state.config.lock().unwrap() = cfg.clone();
                cfg.save();
            }
            ui.add_space(12.0);
        }
    });
}

fn theme_swatch(ui: &mut Ui, kind: ThemeKind, selected: bool) -> bool {
    let preview = Theme::from_kind(kind);
    let width = 138.0;
    let height = 98.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, height), Sense::click());
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::same(14.0), preview.card);

    // Vertical gradient preview (mini window)
    let preview_rect = Rect::from_min_size(
        rect.min + egui::vec2(10.0, 10.0),
        egui::vec2(width - 20.0, 46.0),
    );
    // Approximate gradient using 3 horizontal strips
    let strips = [preview.bg_top, preview.bg_mid, preview.bg_bot];
    let strip_h = preview_rect.height() / strips.len() as f32;
    for (i, c) in strips.iter().enumerate() {
        let r = Rect::from_min_size(
            egui::pos2(preview_rect.min.x, preview_rect.min.y + strip_h * i as f32),
            egui::vec2(preview_rect.width(), strip_h),
        );
        painter.rect_filled(r, Rounding::ZERO, *c);
    }
    // Overlay accent dots
    let dots = [preview.teal, preview.coral, preview.yellow, preview.purple];
    for (i, c) in dots.iter().enumerate() {
        let cx = preview_rect.min.x + 12.0 + i as f32 * 20.0;
        let cy = preview_rect.center().y;
        painter.circle_filled(egui::pos2(cx, cy), 5.0, *c);
    }
    // Rounded mask on preview corners
    painter.rect_stroke(
        preview_rect,
        Rounding::same(6.0),
        Stroke::new(1.0, preview.card_outline),
    );

    // Name
    painter.text(
        egui::pos2(rect.min.x + 14.0, rect.max.y - 20.0),
        Align2::LEFT_CENTER,
        kind.label(),
        f_sb(13.0),
        preview.text,
    );

    // Border
    let stroke = if selected {
        Stroke::new(2.0, preview.teal)
    } else if resp.hovered() {
        Stroke::new(1.0, preview.card_outline)
    } else {
        Stroke::new(1.0, preview.card_outline.linear_multiply(0.6))
    };
    painter.rect_stroke(rect, Rounding::same(14.0), stroke);

    if selected {
        let cx = rect.max.x - 14.0;
        let cy = rect.min.y + 14.0;
        painter.circle_filled(egui::pos2(cx, cy), 8.0, preview.teal);
        painter.text(
            egui::pos2(cx, cy),
            Align2::CENTER_CENTER,
            "✓",
            f_bold(10.0),
            preview.on_accent(),
        );
    }

    resp.clicked()
}

fn toggle_row(
    ui: &mut Ui,
    glyph: &str,
    label: &str,
    desc: &str,
    value: &mut bool,
    theme: &Theme,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.add_sized(
            [28.0, 28.0],
            egui::Label::new(RichText::new(glyph).size(16.0)),
        );
        ui.vertical(|ui| {
            ui.label(RichText::new(label).font(f_med(13.0)).color(theme.text));
            ui.label(RichText::new(desc).font(f_reg(10.5)).color(theme.text_dim));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let before = *value;
            toggle_switch(ui, value, theme);
            if *value != before {
                changed = true;
            }
        });
    });
    ui.add_space(12.0);
    changed
}

fn toggle_switch(ui: &mut Ui, value: &mut bool, theme: &Theme) {
    let size = egui::vec2(38.0, 22.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    if resp.clicked() {
        *value = !*value;
    }
    let painter = ui.painter();
    let bg = if *value { theme.teal } else { theme.pill_track };
    painter.rect_filled(rect, Rounding::same(999.0), bg);
    let knob_r = 8.0;
    let knob_x = if *value {
        rect.max.x - knob_r - 3.0
    } else {
        rect.min.x + knob_r + 3.0
    };
    let knob_center = egui::pos2(knob_x, rect.center().y);
    let knob_color = if *value {
        theme.on_accent()
    } else {
        theme.text_muted
    };
    painter.circle_filled(knob_center, knob_r, knob_color);
}

fn interval_row(
    ui: &mut Ui,
    glyph: &str,
    label: &str,
    desc: &str,
    value: &mut u64,
    theme: &Theme,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.add_sized(
            [28.0, 28.0],
            egui::Label::new(RichText::new(glyph).size(16.0)),
        );
        ui.vertical(|ui| {
            ui.label(RichText::new(label).font(f_med(13.0)).color(theme.text));
            ui.label(RichText::new(desc).font(f_reg(10.5)).color(theme.text_dim));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let options = [1u64, 3, 5, 10, 30];
            for opt in options.iter().rev() {
                let selected = *value == *opt;
                let color = if selected {
                    theme.teal
                } else {
                    theme.text_muted
                };
                let bg = if selected {
                    theme.card_strong
                } else {
                    theme.pill_track
                };
                let btn = egui::Button::new(
                    RichText::new(format!("{opt}s"))
                        .font(f_sb(11.0))
                        .color(color),
                )
                .fill(bg)
                .rounding(Rounding::same(999.0))
                .stroke(Stroke::NONE);
                if ui.add(btn).clicked() && !selected {
                    *value = *opt;
                    changed = true;
                }
                ui.add_space(4.0);
            }
        });
    });
    ui.add_space(12.0);
    changed
}

fn info_row(ui: &mut Ui, glyph: &str, label: &str, value: &str, theme: &Theme) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [28.0, 24.0],
            egui::Label::new(RichText::new(glyph).size(15.0)),
        );
        ui.label(RichText::new(label).font(f_med(12.5)).color(theme.text));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(value)
                    .font(f_mono(11.0))
                    .color(theme.text_muted),
            );
        });
    });
    ui.add_space(10.0);
}
