/// Offers an implementation of 'MathFont' for fonts derived from 'ttfparser' crate.
#[cfg(feature="ttfparser-fontparser")]
pub mod ttf_parser;

/// Offers an implementation of 'MathFont' for fonts derived from 'font' crate.
#[cfg(feature="fontrs-fontparser")]
pub mod font;