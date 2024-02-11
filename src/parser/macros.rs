//! Structure for custom macros (as created by e.g. `\newcommand{..}`)




/// A collection of custom commands. You can find a macro with the given name using [`CommandCollection::query`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CommandCollection(Vec<(String, CustomCommand)>);



impl CommandCollection {
    /// Creates a new empty [`CommandCollection`]    
    pub const fn new() -> Self {
        Self(Vec::new())
    }
}




/// A custom LateX command, as defined by e.g. \newcommand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomCommand {

}

impl CustomCommand {

}




