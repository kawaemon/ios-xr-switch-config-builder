use js_sys::{Array, RegExp as JsRegExp};
use wasm_bindgen::JsCast;

pub struct Regex {
    pattern: &'static str,
}

impl Regex {
    pub const fn new(pattern: &'static str) -> Self {
        Self { pattern }
    }

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

pub struct Captures {
    groups: Vec<Option<String>>,
}

impl Captures {
    pub fn get(&self, index: usize) -> Option<Match> {
        self.groups
            .get(index)?
            .as_ref()
            .map(|s| Match { text: s.clone() })
    }
}

pub struct Match {
    text: String,
}

impl Match {
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

#[macro_export]
macro_rules! regex {
    ($pattern:expr) => {{
        static RE: $crate::regex::Regex = $crate::regex::Regex::new($pattern);
        &RE
    }};
}
