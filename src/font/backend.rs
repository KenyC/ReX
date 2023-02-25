


#[cfg(feature="ttfparser-backend")]
pub mod ttf_parser {

    use ttf_parser::GlyphId;


    pub struct MathFont {
    }

    pub struct MathHeader<'a>(ttf_parser::math::Table<'a>);

    impl<'a> MathHeader<'a> {
        fn safe_italics(&self, glyph_id : u16) -> Option<i16> {
            let value = self.0.glyph_info?
                .italic_corrections?
                .get(GlyphId(glyph_id))?
                .value;
            Some(value)
        }

        fn safe_attachment(&self, glyph_id : u16) -> Option<i16> {
            let value = self.0.glyph_info?
                .top_accent_attachments?
                .get(GlyphId(glyph_id))?
                .value;
            Some(value)
        }
    }



    impl<'a> crate::font::IsMathHeader for MathHeader<'a> {
        fn italics(&self, glyph_id : u16) -> i16 {
            self.safe_italics(glyph_id).unwrap_or_default()
        }

        fn attachment(&self, glyph_id : u16) -> i16 {
            self.safe_attachment(glyph_id).unwrap_or_default()
        }

        fn horz_variant(&self, gid: u32, width: crate::dimensions::Length<crate::dimensions::Font>) -> font::opentype::math::assembly::VariantGlyph {
            todo!()
        }

        fn vert_variant(&self, gid: u32, height: crate::dimensions::Length<crate::dimensions::Font>) -> font::opentype::math::assembly::VariantGlyph {
            todo!()
        }
    }

}