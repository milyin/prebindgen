//! Resolved logical paths carried directly by boundary wires.
/// Typed step identifying a logical component independently of generated names.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentSegment {
    /// A source record field.
    Field(String),
    /// A constructor argument.
    Argument(String),
    /// An input selector arm.
    Alternative(usize),
    /// One simultaneous output view.
    View(usize),
    /// A source getter's result.
    Getter(String),
    /// An input selector's discriminator.
    Selector,
    /// An adapter-defined logical component with no implied source projection.
    Named(String),
}
impl ComponentSegment {
    /// Default presentation name; applications may assign a different name in the schema.
    pub fn name(&self) -> String {
        match self {
            Self::Field(name) | Self::Argument(name) | Self::Getter(name) | Self::Named(name) => {
                name.clone()
            }
            Self::Alternative(index) | Self::View(index) => index.to_string(),
            Self::Selector => "selector".into(),
        }
    }
}

/// Logical component identity, including nesting. Several wires may implement it.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentPath(pub Vec<ComponentSegment>);
impl ComponentPath {
    /// Prefix a component with a containing field, view, or alternative.
    pub fn prepend(&mut self, segment: ComponentSegment) {
        self.0.insert(0, segment);
    }
}
