#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol(String);

impl Symbol {
    pub fn parse(raw: &str) -> Option<Self> {
        let s = raw.trim().to_ascii_uppercase();
        let ok = !s.is_empty()
            && s.len() <= 12
            && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
        ok.then_some(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}