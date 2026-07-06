use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontId, TextStyle};

pub const INTER_LIGHT: &str = "inter-light";
pub const INTER_REGULAR: &str = "inter-regular";
pub const INTER_MEDIUM: &str = "inter-medium";
pub const INTER_SEMIBOLD: &str = "inter-semibold";
pub const INTER_BOLD: &str = "inter-bold";
pub const JETBRAINS: &str = "jetbrains-mono";
pub const SPACE_GROTESK: &str = "space-grotesk";
pub const INSTRUMENT_SERIF: &str = "instrument-serif";
pub const INSTRUMENT_ITALIC: &str = "instrument-italic";

/// Named font families reachable from any widget via [`FontId::new`].
pub mod fam {
    use eframe::egui::FontFamily;
    pub fn light() -> FontFamily {
        FontFamily::Name("inter-light".into())
    }
    pub fn regular() -> FontFamily {
        FontFamily::Proportional
    }
    pub fn medium() -> FontFamily {
        FontFamily::Name("inter-medium".into())
    }
    pub fn semibold() -> FontFamily {
        FontFamily::Name("inter-semibold".into())
    }
    pub fn bold() -> FontFamily {
        FontFamily::Name("inter-bold".into())
    }
    pub fn mono() -> FontFamily {
        FontFamily::Monospace
    }
    pub fn space_grotesk() -> FontFamily {
        FontFamily::Name("space-grotesk".into())
    }
    pub fn instrument() -> FontFamily {
        FontFamily::Name("instrument-serif".into())
    }
    pub fn instrument_italic() -> FontFamily {
        FontFamily::Name("instrument-italic".into())
    }
}

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    macro_rules! add {
        ($name:expr, $bytes:expr) => {
            fonts
                .font_data
                .insert($name.to_owned(), FontData::from_static($bytes));
        };
    }

    add!(INTER_LIGHT, include_bytes!("../assets/Inter-Light.ttf"));
    add!(INTER_REGULAR, include_bytes!("../assets/Inter-Regular.ttf"));
    add!(INTER_MEDIUM, include_bytes!("../assets/Inter-Medium.ttf"));
    add!(
        INTER_SEMIBOLD,
        include_bytes!("../assets/Inter-SemiBold.ttf")
    );
    add!(INTER_BOLD, include_bytes!("../assets/Inter-Bold.ttf"));
    add!(
        JETBRAINS,
        include_bytes!("../assets/JetBrainsMono-Regular.ttf")
    );
    add!(
        SPACE_GROTESK,
        include_bytes!("../assets/SpaceGrotesk-Variable.ttf")
    );
    add!(
        INSTRUMENT_SERIF,
        include_bytes!("../assets/InstrumentSerif-Regular.ttf")
    );
    add!(
        INSTRUMENT_ITALIC,
        include_bytes!("../assets/InstrumentSerif-Italic.ttf")
    );

    // Proportional = Inter Regular (with lighter/heavier as fallbacks)
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, INTER_REGULAR.into());

    // Monospace = JetBrains Mono
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, JETBRAINS.into());

    // Collect the existing emoji fallback fonts from the default Proportional
    // family so every named family can render emoji glyphs too.
    let emoji_fallbacks: Vec<String> = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|n| n != INTER_REGULAR)
        .collect();

    // Named families to reach specific weights, with emoji fallback
    for (name, key) in [
        ("inter-light", INTER_LIGHT),
        ("inter-medium", INTER_MEDIUM),
        ("inter-semibold", INTER_SEMIBOLD),
        ("inter-bold", INTER_BOLD),
        ("space-grotesk", SPACE_GROTESK),
        ("instrument-serif", INSTRUMENT_SERIF),
        ("instrument-italic", INSTRUMENT_ITALIC),
    ] {
        let mut list = vec![key.to_owned(), INTER_REGULAR.to_owned()];
        list.extend(emoji_fallbacks.iter().cloned());
        fonts.families.insert(FontFamily::Name(name.into()), list);
    }

    ctx.set_fonts(fonts);

    // Base text styles — tuned to match the Aurora mockup rhythm.
    ctx.style_mut(|s| {
        use TextStyle::*;
        s.text_styles
            .insert(Heading, FontId::new(20.0, fam::bold()));
        s.text_styles
            .insert(Body, FontId::new(13.0, fam::regular()));
        s.text_styles
            .insert(Button, FontId::new(12.5, fam::semibold()));
        s.text_styles
            .insert(Small, FontId::new(10.5, fam::regular()));
        s.text_styles
            .insert(Monospace, FontId::new(11.0, fam::mono()));
    });
}
