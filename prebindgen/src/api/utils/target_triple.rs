use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;
use target_lexicon::{OperatingSystem, Triple};

/// TargetTriple is a small utility around `target_lexicon::Triple` with helpers
/// to access parts and to convert into Rust cfg tokens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetTriple(Triple);

impl TargetTriple {
    /// Parse from a string like "aarch64-apple-darwin".
    pub fn parse(s: &str) -> Result<Self, String> {
        s.parse::<Triple>()
            .map(TargetTriple)
            .map_err(|e| format!("Failed to parse target triple '{s}': {e}"))
    }

    /// Create from an existing target_lexicon Triple.
    pub fn from_triple(triple: Triple) -> Self {
        Self(triple)
    }

    /// Get the architecture as a canonical string used by Rust cfg target_arch.
    pub fn arch(&self) -> String {
        self.0.architecture.to_string()
    }

    /// Get the vendor as string used by Rust cfg target_vendor.
    pub fn vendor(&self) -> String {
        self.0.vendor.to_string()
    }

    /// Get the operating system as string used by Rust cfg target_os.
    /// Maps Darwin to "macos" to match Rust cfg semantics.
    pub fn os(&self) -> String {
        match self.0.operating_system {
            OperatingSystem::Darwin(_) => "macos".to_string(),
            ref os => os.to_string(),
        }
    }

    /// Get the environment as string used by Rust cfg target_env (may be "unknown").
    pub fn env(&self) -> Option<String> {
        if self.0.environment == target_lexicon::Environment::Unknown {
            None
        } else {
            Some(self.0.environment.to_string())
        }
    }

    /// Access the inner Triple.
    pub fn as_triple(&self) -> &Triple {
        &self.0
    }

    /// Decompose into the inner Triple.
    pub fn into_triple(self) -> Triple {
        self.0
    }

    /// Build a cfg expression TokenStream like:
    /// all(target_arch = "aarch64", target_vendor = "apple", target_os = "macos", target_env = "gnu")
    /// Omits target_env when unknown/empty.
    pub fn to_cfg_tokens(&self) -> TokenStream {
        let arch = LitStr::new(&self.arch(), proc_macro2::Span::call_site());
        let vendor = LitStr::new(&self.vendor(), proc_macro2::Span::call_site());
        let os = LitStr::new(&self.os(), proc_macro2::Span::call_site());
        let env = self.env();
        let mut parts: Vec<TokenStream> = Vec::with_capacity(4);
        parts.push(quote! { target_arch = #arch });
        parts.push(quote! { target_vendor = #vendor });
        parts.push(quote! { target_os = #os });
        if let Some(env) = env {
            let env_lit = LitStr::new(&env, proc_macro2::Span::call_site());
            parts.push(quote! { target_env = #env_lit });
        }
        if parts.len() == 1 {
            parts.remove(0)
        } else {
            quote! { all( #(#parts),* ) }
        }
    }
}

impl std::str::FromStr for TargetTriple {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TargetTriple::parse(s)
    }
}

/// Allow quoting a TargetTriple directly, yielding its cfg tokens.
impl quote::ToTokens for TargetTriple {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_cfg_tokens());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_darwin_to_macos() {
        let tt = TargetTriple::parse("aarch64-apple-darwin").unwrap();
        assert_eq!(tt.os(), "macos");
    }

    #[test]
    fn builds_cfg_without_unknown_env() {
        let tt = TargetTriple::parse("x86_64-unknown-linux-gnu").unwrap();
        let ts = tt.to_cfg_tokens().to_string();
        assert!(ts.contains("target_arch = \"x86_64\""));
        assert!(ts.contains("target_vendor = \"unknown\""));
        assert!(ts.contains("target_os = \"linux\""));
        assert!(ts.contains("target_env = \"gnu\""));
    }
}
