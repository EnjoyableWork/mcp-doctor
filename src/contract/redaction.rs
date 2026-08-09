use std::fmt;

use serde::{Serialize, Serializer};

pub(super) const REDACTION_MARKER: &str = "[REDACTED]";

/// Owns an untrusted value without allowing ordinary formatting or serialization
/// to reveal it.
pub(super) struct Sensitive<T>(T);

impl<T> Sensitive<T> {
    pub(super) fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> Sensitive<T>
where
    T: AsRef<[u8]>,
{
    pub(super) fn byte_count(&self) -> usize {
        self.0.as_ref().len()
    }

    pub(super) fn redacted(&self) -> RedactedValue {
        RedactedValue::new(self.byte_count())
    }
}

impl<T> fmt::Debug for Sensitive<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(REDACTION_MARKER)
    }
}

impl<T> fmt::Display for Sensitive<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(REDACTION_MARKER)
    }
}

impl<T> Serialize for Sensitive<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(REDACTION_MARKER)
    }
}

/// Structural evidence that a value was observed without retaining or
/// exposing the value itself.
#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct RedactedValue {
    byte_count: usize,
}

impl RedactedValue {
    pub(super) const fn new(byte_count: usize) -> Self {
        Self { byte_count }
    }

    pub(super) const fn byte_count(self) -> usize {
        self.byte_count
    }
}

impl fmt::Display for RedactedValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{REDACTION_MARKER} ({} bytes)", self.byte_count)
    }
}

#[cfg(test)]
mod tests {
    use super::{REDACTION_MARKER, Sensitive};

    const SENTINEL: &str = "synthetic-private-value-never-report-7f2c";

    #[test]
    fn sensitive_values_redact_debug_display_and_json() {
        let sensitive = Sensitive::new(SENTINEL.to_owned());

        let rendered = [
            format!("{sensitive:?}"),
            sensitive.to_string(),
            serde_json::to_string(&sensitive).expect("redaction marker should serialize"),
        ]
        .join("\n");

        assert!(rendered.contains(REDACTION_MARKER));
        assert!(
            !rendered.contains(SENTINEL),
            "ordinary formatting must not reveal a synthetic sentinel"
        );
        assert_eq!(sensitive.byte_count(), SENTINEL.len());
    }
}
