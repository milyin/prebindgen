use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

/// TargetTriple is a small utility around `target_lexicon::Triple` with helpers
/// to access parts and to convert into Rust cfg tokens.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TargetTriple {
    triple: String,
    arch: Option<String>,
    vendor: Option<String>,
    os: Option<String>,
    env: Option<String>,
}

fn extract_cfg_condition(s: &str, name: &str) -> Option<String> {
    let prefix = format!("{}=\"", name);
    if s.starts_with(&prefix) && s.ends_with('"') {
        let s = &s[prefix.len()..s.len() - 1];
        if s.is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    } else {
        None
    }
}

impl TargetTriple {
    /// Parse from a string like "aarch64-apple-darwin".
    pub fn parse(s: impl Into<String>) -> Result<Self, String> {
        let triple = s.into();
        // run `rustc --print cfg --target <target_triple>` and capture the output
        let output = std::process::Command::new("rustc")
            .args(["--print", "cfg", "--target", &triple])
            .output()
            .map_err(|e| format!("Failed to run rustc: {e}"))?;

        if !output.status.success() {
            return Err(format!("rustc failed with status: {}", output.status));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut arch = None;
        let mut vendor = None;
        let mut os = None;
        let mut env = None;

        for line in stdout.lines() {
            if let Some(a) = extract_cfg_condition(line, "target_arch") {
                arch = Some(a);
            } else if let Some(v) = extract_cfg_condition(line, "target_vendor") {
                vendor = Some(v);
            } else if let Some(o) = extract_cfg_condition(line, "target_os") {
                os = Some(o);
            } else if let Some(e) = extract_cfg_condition(line, "target_env") {
                env = Some(e);
            }
        }

        Ok(Self {
            triple,
            arch,
            vendor,
            os,
            env,
        })
    }

    /// Get the architecture as a canonical string used by Rust cfg target_arch.
    pub fn arch(&self) -> Option<&str> {
        self.arch.as_deref()
    }

    /// Get the vendor as string used by Rust cfg target_vendor.
    pub fn vendor(&self) -> Option<&str> {
        self.vendor.as_deref()
    }

    /// Get the operating system as string used by Rust cfg target_os.
    pub fn os(&self) -> Option<&str> {
        self.os.as_deref()
    }

    /// Get the environment as string used by Rust cfg target_env (may be "unknown").
    pub fn env(&self) -> Option<&str> {
        self.env.as_deref()
    }

    /// Build a cfg expression TokenStream like:
    /// all(target_arch = "aarch64", target_vendor = "apple", target_os = "macos", target_env = "gnu")
    /// Omits target_env when unknown/empty.
    pub fn to_cfg_tokens(&self) -> TokenStream {
        let mut parts: Vec<TokenStream> = Vec::with_capacity(4);
        if let Some(ref arch) = self.arch {
            let arch = LitStr::new(arch, proc_macro2::Span::call_site());
            parts.push(quote! { target_arch = #arch });
        }
        if let Some(ref vendor) = self.vendor {
            let vendor = LitStr::new(vendor, proc_macro2::Span::call_site());
            parts.push(quote! { target_vendor = #vendor });
        }
        if let Some(ref os) = self.os {
            let os = LitStr::new(os, proc_macro2::Span::call_site());
            parts.push(quote! { target_os = #os });
        }
        if let Some(ref env) = self.env {
            let env = LitStr::new(env, proc_macro2::Span::call_site());
            parts.push(quote! { target_env = #env });
        }
        if parts.is_empty() {
            quote! { true }
        } else if parts.len() == 1 {
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
    fn aarch64_apple_darwin() {
        let tt = TargetTriple::parse("aarch64-apple-darwin").unwrap();
        assert!(
            tt.arch() == Some("aarch64"),
            "Unexpected architecture found {:?}",
            tt.arch()
        );
        assert!(
            tt.vendor() == Some("apple"),
            "Unexpected vendor found {:?}",
            tt.vendor()
        );
        assert!(
            tt.os() == Some("macos"),
            "Unexpected OS found {:?}",
            tt.os()
        );
        assert!(
            tt.env().is_none(),
            "Unexpected environment found {:?}",
            tt.env()
        );
    }

    #[test]
    fn x86_64_unknown_linux() {
        let tt = TargetTriple::parse("x86_64-unknown-linux-gnu").unwrap();
        assert!(
            tt.arch() == Some("x86_64"),
            "Unexpected architecture found {:?}",
            tt.arch()
        );
        assert!(
            tt.vendor() == Some("unknown"),
            "Unexpected vendor found {:?}",
            tt.vendor()
        );
        assert!(
            tt.os() == Some("linux"),
            "Unexpected OS found {:?}",
            tt.os()
        );
        assert!(
            tt.env() == Some("gnu"),
            "Unexpected environment found {:?}",
            tt.env()
        );
    }

    #[test]
    fn armv7_unknown_linux_gnueabihf() {
        let tt = TargetTriple::parse("armv7-unknown-linux-gnueabihf").unwrap();
        assert!(
            tt.arch() == Some("arm"),
            "Unexpected architecture found {:?}",
            tt.arch()
        );
        assert!(
            tt.vendor() == Some("unknown"),
            "Unexpected vendor found {:?}",
            tt.vendor()
        );
        assert!(
            tt.os() == Some("linux"),
            "Unexpected OS found {:?}",
            tt.os()
        );
        assert!(
            tt.env() == Some("gnu"),
            "Unexpected environment found {:?}",
            tt.env()
        );
    }
}
