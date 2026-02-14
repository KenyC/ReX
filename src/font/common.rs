use std::convert::{TryFrom, TryInto};

/// The id of a glyph (represented as u16)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct GlyphId(u16);

impl From<u16> for GlyphId {
    fn from(x: u16) -> Self { Self(x) }
}

impl Into<u16> for GlyphId {
    fn into(self) -> u16 { self.0 }
}


/// Extended glyphs (like large '}'):
/// These are formed in one of two ways: the font provides bigger versions of '}' (replacement glyphs)
/// and it also provides a recipe for forming even bigger versions, by assembling some parts together (decomposing '→' into a line and a hook).
#[derive(Debug, Clone)]
pub enum VariantGlyph {
    /// Id for a replacement glyph.
    Replacement(GlyphId),
    /// Instructions on how to form the bigger glyphs and whether it is a horizontal extended glyph (e.g. a long '→') or a vertical extended glyph (e.g. a tall '}').
    Constructable(Direction, Vec<GlyphInstruction>),
}

/// Direction of an extended glyph
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    /// horizontal direction (as for '}')
    Horizontal,
    /// vertical direction (as for '→')
    Vertical
}


/// Specifies the script level at which a glyph is rendered, such as superscript, subscript, or their nested forms (e.g. subscripts of subscripts).
/// This is mainly to select a stylistic variant in `ssty` feature of font (cf [reference]()), i.e. another glyph to draw that will look neater in sub- and superscripts.
#[derive(Debug, Clone, Copy)]
pub enum ScriptLevel {
    /// The glyph is rendered as a first-level script (superscript or subscript).
    LevelOne,
    /// The glyph is rendered as a nested script (e.g., a superscript of a superscript or a subscript of a subscript).
    LevelTwo,
}

/// One part of the extended glyph construction.
/// The different parts are assembled together with some overlap.
#[derive(Debug, Clone, Copy)]
pub struct GlyphInstruction {
    /// The id of the part 
    pub gid: GlyphId,
    /// How much it overlaps with the previous glyph part.\
    /// For instance, when drawing '}', the first piece will have a certain height `h` and we will strart drawing the second part at `h - overlap`
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
