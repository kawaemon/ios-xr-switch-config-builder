// Lightweight regex abstraction.
// wasm32: uses JS RegExp.
// host (tests on x86_64): uses regex crate, only for non-wasm targets.

#[cfg(target_arch = "wasm32")]
mod imp {
    use js_sys::{Array, RegExp as JsRegExp};
    use wasm_bindgen::JsCast;

    /// Lightweight wrapper around JavaScript `RegExp` used in WASM.
    pub struct Regex {
        pattern: &'static str,
    }

    impl Regex {
        /// Create a new regex from a static pattern.
        pub const fn new(pattern: &'static str) -> Self {
            Self { pattern }
        }

        /// Execute the regex against the given text and return captures.
        pub fn captures(&self, text: &str) -> Option<Captures> {
            let re = JsRegExp::new(self.pattern, "");
            let result = re.exec(text)?;
            let arr: Array = result.dyn_into().ok()?;

            let mut groups = Vec::new();
            for i in 0..arr.length() {
                let val = arr.get(i);
                if val.is_undefined() || val.is_null() {
                    groups.push(None);
                } else {
                    groups.push(val.as_string());
                }
            }

            Some(Captures { groups })
        }
    }

    /// Collection of captured groups from a regex match.
    pub struct Captures {
        groups: Vec<Option<String>>,
    }

    impl Captures {
        /// Get a specific capture group.
        pub fn get(&self, index: usize) -> Option<Match> {
            self.groups
                .get(index)?
                .as_ref()
                .map(|s| Match { text: s.clone() })
        }
    }

    /// Single capture group match.
    pub struct Match {
        text: String,
    }

    impl Match {
        /// Borrow the matched text.
        pub fn as_str(&self) -> &str {
            &self.text
        }
    }

    pub use Regex as RegexImpl;
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use regex::Regex as HostRegex;

    /// Host-side regex wrapper used when not targeting WASM.
    pub struct Regex {
        inner: HostRegex,
    }

    impl Regex {
        /// Create a compiled regex from a static pattern.
        pub fn new(pattern: &'static str) -> Self {
            let inner = HostRegex::new(pattern).expect("regex pattern must compile");
            Self { inner }
        }

        /// Execute the regex and return captures.
        pub fn captures(&self, text: &str) -> Option<Captures> {
            self.inner.captures(text).map(|caps| Captures {
                groups: caps
                    .iter()
                    .map(|m| m.map(|m| m.as_str().to_string()))
                    .collect(),
            })
        }
    }

    /// Collection of captured groups from a regex match.
    pub struct Captures {
        groups: Vec<Option<String>>,
    }

    impl Captures {
        /// Get a specific capture group.
        pub fn get(&self, index: usize) -> Option<Match> {
            self.groups
                .get(index)?
                .as_ref()
                .map(|s| Match { text: s.clone() })
        }
    }

    /// Single capture group match.
    pub struct Match {
        text: String,
    }

    impl Match {
        /// Borrow the matched text.
        pub fn as_str(&self) -> &str {
            &self.text
        }
    }

    pub use Regex as RegexImpl;
}

pub use imp::RegexImpl as Regex;

#[macro_export]
macro_rules! regex {
    ($pattern:expr) => {{
        static RE: std::sync::OnceLock<$crate::regex::Regex> = std::sync::OnceLock::new();
        RE.get_or_init(|| $crate::regex::Regex::new($pattern))
    }};
}
