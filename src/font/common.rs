#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct GlyphId(pub u32);

impl From<font::GlyphId> for GlyphId {
	#[inline]
    fn from(glyph_id: font::GlyphId) -> Self {
        Self(glyph_id.0)
    }
}

impl Into<font::GlyphId> for GlyphId {
    fn into(self) -> font::GlyphId {
    	font::GlyphId(self.0)
    }
}

