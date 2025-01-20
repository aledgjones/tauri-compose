use skia_safe::{FontMgr, Typeface};

pub struct Fonts {
    pub bravura: Typeface,
    pub libre_baskerville: Typeface,
}

impl Fonts {
    pub fn new() -> Self {
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

        Fonts {
            bravura,
            libre_baskerville,
        }
    }
}
