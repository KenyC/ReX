macro_rules! accent {
    ($sym:expr, $nucleus:expr) => (
        ParseNode::Accent(
            Accent {
                symbol: $sym,
                nucleus: $nucleus
            }
        )
    )
}

