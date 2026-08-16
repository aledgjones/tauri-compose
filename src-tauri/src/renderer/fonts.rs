use lazy_static::lazy_static;
use skia_safe::{FontMgr, Typeface};

pub struct Fonts {
    pub bravura: Typeface,
    pub libre_baskerville: Typeface,
    pub system_sans: Typeface,
}

impl Fonts {
    fn new() -> Self {
        let font_mgr = FontMgr::new();

        // Load the fixed set of fonts
        let bravura_font_data = include_bytes!("../../../src/assets/fonts/bravura.woff2");
        let libre_baskerville_font_data =
            include_bytes!("../../../src/assets/fonts/libre-baskerville.woff2");

        // Load the typefaces
        let bravura = font_mgr
            .new_from_data(bravura_font_data, None)
            .expect("Failed to load Bravura font");

        let libre_baskerville = font_mgr
            .new_from_data(libre_baskerville_font_data, None)
            .expect("Failed to load Libre Baskerville font");
        let system_sans = font_mgr
            .match_family_style("Helvetica", Default::default())
            .expect("Failed to load the system sans-serif font");

        Fonts {
            bravura,
            libre_baskerville,
            system_sans,
        }
    }
}

lazy_static! {
    pub static ref FONTS: Fonts = Fonts::new();
}

#[cfg(test)]
mod tests {
    use super::FONTS;

    #[test]
    fn system_sans_resolves_to_helvetica() {
        assert!(FONTS.system_sans.family_name().contains("Helvetica"));
    }
}
