//! Attribute related module. This contains a wrapper to escape values.

const CONTENT_ESCAPE: [char; 6] = ['&', '<', '>', '"', '\'', '/'];

/// Wrapper around a [str] that will escape the content when writing.
///
/// This implementation will transform:
/// - `&` to `&amp;`
/// - `<` to `&lt;`
/// - `>` to `&gt;`
/// - `"` to `&quot;`
/// - `'` to `&#x27;`
/// - `/` to `&#x2F;`
pub struct EscapedContent<'a>(pub &'a str);

impl std::fmt::Display for EscapedContent<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }
        let mut start: usize = 0;
        while let Some(index) = self.0[start..].find(CONTENT_ESCAPE) {
            if index > 0 {
                f.write_str(&self.0[start..(start + index)])?;
            }
            let begin = start + index;
            debug_assert!(start <= begin);
            let end = begin + 1;
            debug_assert!(begin < self.0.len());
            debug_assert!(begin < end);
            debug_assert!(end <= self.0.len());
            match &self.0[begin..end] {
                "&" => f.write_str("&amp;")?,
                "<" => f.write_str("&lt;")?,
                ">" => f.write_str("&gt;")?,
                "\"" => f.write_str("&quot;")?,
                "'" => f.write_str("&#x27;")?,
                "/" => f.write_str("&#x2F;")?,
                other => f.write_str(other)?,
            };
            start = end;
            debug_assert!(start <= self.0.len());
        }
        f.write_str(&self.0[start..])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test_case::test_case("hello world", "hello world"; "without character to escape")]
    #[test_case::test_case("a\"b", "a&quot;b"; "with special in the middle")]
    #[test_case::test_case("\"a", "&quot;a"; "with special at the beginning")]
    #[test_case::test_case("a\"", "a&quot;"; "with special at the end")]
    fn escaping_content(input: &str, expected: &str) {
        assert_eq!(format!("{}", super::EscapedContent(input)), expected);
    }
}
