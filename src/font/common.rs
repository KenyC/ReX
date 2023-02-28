#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct GlyphId(pub u32);



#[derive(Debug, Clone)]
pub enum VariantGlyph {
    Replacement(u16),
    Constructable(Direction, Vec<GlyphInstruction>),
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Horizontal,
    Vertical
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphInstruction {
    pub gid: u16,
    pub overlap: u16,
}




#[cfg(feature = "fontcrate-backend")]
impl From<font::GlyphId> for GlyphId {
    #[inline]
    fn from(glyph_id: font::GlyphId) -> Self {
        Self(glyph_id.0)
    }
}

#[cfg(feature = "fontcrate-backend")]
impl Into<font::GlyphId> for GlyphId {
    fn into(self) -> font::GlyphId {
        font::GlyphId(self.0)
    }
}



#[cfg(feature = "fontcrate-backend")]
impl From<font::opentype::math::assembly::Direction> for Direction {
    #[inline]
    fn from(from: font::opentype::math::assembly::Direction) -> Self {
        match from {
            font::opentype::math::assembly::Direction::Horizontal => Self::Horizontal,
            font::opentype::math::assembly::Direction::Vertical   => Self::Vertical,
        }
    }
}

#[cfg(feature = "fontcrate-backend")]
impl Into<font::opentype::math::assembly::Direction> for Direction {
    fn into(self) -> font::opentype::math::assembly::Direction {
        match self {
            Self::Horizontal => font::opentype::math::assembly::Direction::Horizontal, 
            Self::Vertical   => font::opentype::math::assembly::Direction::Vertical,   
        }
    }
}


#[cfg(feature = "fontcrate-backend")]
impl From<font::opentype::math::assembly::GlyphInstruction> for GlyphInstruction {
    #[inline]
    fn from(from: font::opentype::math::assembly::GlyphInstruction) -> Self {
        Self { 
            gid:     from.gid, 
            overlap: from.overlap, 
        }
    }
}

#[cfg(feature = "fontcrate-backend")]
impl Into<font::opentype::math::assembly::GlyphInstruction> for GlyphInstruction {
    fn into(self) -> font::opentype::math::assembly::GlyphInstruction {
        font::opentype::math::assembly::GlyphInstruction {
            gid:     self.gid, 
            overlap: self.overlap, 
        }
    }
}


#[cfg(feature = "fontcrate-backend")]
impl From<font::opentype::math::assembly::VariantGlyph> for VariantGlyph {
    #[inline]
    fn from(from: font::opentype::math::assembly::VariantGlyph) -> Self {
        match from {
            font::opentype::math::assembly::VariantGlyph::Replacement(gid) => {
                Self::Replacement(gid)
            },
            font::opentype::math::assembly::VariantGlyph::Constructable(dir, instrs) => {
                Self::Constructable(dir.into(), instrs.into_iter().map(GlyphInstruction::from).collect())
            },
        }
    }
}

#[cfg(feature = "fontcrate-backend")]
impl Into<font::opentype::math::assembly::VariantGlyph> for VariantGlyph {
    fn into(self) -> font::opentype::math::assembly::VariantGlyph {
        match self {
            Self::Replacement(gid) => {
                font::opentype::math::assembly::VariantGlyph::Replacement(gid)
            },
            Self::Constructable(dir, instrs) => {
                font::opentype::math::assembly::VariantGlyph::Constructable(dir.into(), instrs.into_iter().map(GlyphInstruction::into).collect())
            },
        }

    }
}
