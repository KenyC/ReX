use std::convert::{TryFrom, TryInto};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct GlyphId(u16);

impl From<u16> for GlyphId {
    fn from(x: u16) -> Self { Self(x) }
}

impl Into<u16> for GlyphId {
    fn into(self) -> u16 { self.0 }
}



#[derive(Debug, Clone)]
pub enum VariantGlyph {
    Replacement(GlyphId),
    Constructable(Direction, Vec<GlyphInstruction>),
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Horizontal,
    Vertical
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphInstruction {
    pub gid: GlyphId,
    pub overlap: u16,
}




#[cfg(feature="fontrs-fontparser")]
impl TryFrom<font::GlyphId> for GlyphId {
    type Error = <u16 as TryFrom<u32>>::Error;

    #[inline]
    fn try_from(glyph_id: font::GlyphId) -> Result<Self, <u16 as TryFrom<u32>>::Error> {
        Ok(Self(glyph_id.0.try_into()?))
    }
}

#[cfg(feature="fontrs-fontparser")]
impl Into<font::GlyphId> for GlyphId {
    fn into(self) -> font::GlyphId {
        font::GlyphId(self.0.into())
    }
}



#[cfg(feature="ttfparser-fontparser")]
impl From<ttf_parser::GlyphId> for GlyphId {

    #[inline]
    fn from(glyph_id: ttf_parser::GlyphId) -> Self {
        Self(glyph_id.0)
    }
}

#[cfg(feature="ttfparser-fontparser")]
impl Into<ttf_parser::GlyphId> for GlyphId {
    fn into(self) -> ttf_parser::GlyphId {
        ttf_parser::GlyphId(self.0.into())
    }
}



#[cfg(feature="fontrs-fontparser")]
impl From<font::opentype::math::assembly::Direction> for Direction {
    #[inline]
    fn from(from: font::opentype::math::assembly::Direction) -> Self {
        match from {
            font::opentype::math::assembly::Direction::Horizontal => Self::Horizontal,
            font::opentype::math::assembly::Direction::Vertical   => Self::Vertical,
        }
    }
}

#[cfg(feature="fontrs-fontparser")]
impl Into<font::opentype::math::assembly::Direction> for Direction {
    fn into(self) -> font::opentype::math::assembly::Direction {
        match self {
            Self::Horizontal => font::opentype::math::assembly::Direction::Horizontal, 
            Self::Vertical   => font::opentype::math::assembly::Direction::Vertical,   
        }
    }
}


#[cfg(feature="fontrs-fontparser")]
impl From<font::opentype::math::assembly::GlyphInstruction> for GlyphInstruction {
    #[inline]
    fn from(from: font::opentype::math::assembly::GlyphInstruction) -> Self {
        Self { 
            gid:     GlyphId::from(from.gid), 
            overlap: from.overlap, 
        }
    }
}

#[cfg(feature="fontrs-fontparser")]
impl Into<font::opentype::math::assembly::GlyphInstruction> for GlyphInstruction {
    fn into(self) -> font::opentype::math::assembly::GlyphInstruction {
        font::opentype::math::assembly::GlyphInstruction {
            gid:     self.gid.into(), 
            overlap: self.overlap, 
        }
    }
}


#[cfg(feature="fontrs-fontparser")]
impl From<font::opentype::math::assembly::VariantGlyph> for VariantGlyph {
    #[inline]
    fn from(from: font::opentype::math::assembly::VariantGlyph) -> Self {
        match from {
            font::opentype::math::assembly::VariantGlyph::Replacement(gid) => {
                Self::Replacement(GlyphId::from(gid))
            },
            font::opentype::math::assembly::VariantGlyph::Constructable(dir, instrs) => {
                Self::Constructable(dir.into(), instrs.into_iter().map(GlyphInstruction::from).collect())
            },
        }
    }
}

#[cfg(feature="fontrs-fontparser")]
impl Into<font::opentype::math::assembly::VariantGlyph> for VariantGlyph {
    fn into(self) -> font::opentype::math::assembly::VariantGlyph {
        match self {
            Self::Replacement(gid) => {
                font::opentype::math::assembly::VariantGlyph::Replacement(gid.into())
            },
            Self::Constructable(dir, instrs) => {
                font::opentype::math::assembly::VariantGlyph::Constructable(dir.into(), instrs.into_iter().map(GlyphInstruction::into).collect())
            },
        }

    }
}
