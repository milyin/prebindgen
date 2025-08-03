/// Rust edition for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustEdition {
    /// Rust 2021 edition
    Edition2021,
    /// Rust 2024 edition
    Edition2024,
}

impl RustEdition {
    /// Convert edition to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            RustEdition::Edition2021 => "2021",
            RustEdition::Edition2024 => "2024",
        }
    }
}

impl std::fmt::Display for RustEdition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for RustEdition {
    /// Default edition based on compiler version
    fn default() -> Self {
        if_rust_version::if_rust_version! { >= 1.82 {
            RustEdition::Edition2024
        } else {
            RustEdition::Edition2021
        }}
    }
}