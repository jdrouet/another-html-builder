//! Attribute related module. This contains the traits needed to implement a new
//! kind of [AttributeValue] but also a wrapper to escape values.

use std::fmt::{Display, Write};

/// Wrapper around a [str] that will escape the content when writing.
pub struct EscapedValue<'a>(pub &'a str);

impl std::fmt::Display for EscapedValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }
        let mut start: usize = 0;
        while let Some(index) = self.0[start..].find('"') {
            if index > 0 {
                f.write_str(&self.0[start..(start + index)])?;
            }
            f.write_str("\\\"")?;
            let end = start + index + 1;
            debug_assert!(start < end && end <= self.0.len());
            start = end;
        }
        f.write_str(&self.0[start..])?;
        Ok(())
    }
}

macro_rules! attribute_value {
    ($type:ty) => {
        impl AttributeValue for $type {
            fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{self}")
            }
        }
    };
}

/// Represents an element attribute name.
pub trait AttributeName {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl AttributeName for &str {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self)
    }
}

/// Represents an element attribute value.
///
/// This value should be escaped for double quotes for example.
/// The implementation of this trait on `&str` already implements this.
pub trait AttributeValue {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl AttributeValue for &str {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        EscapedValue(self).fmt(f)
    }
}

#[inline]
fn render_attr_name_only<N: AttributeName>(
    f: &mut std::fmt::Formatter<'_>,
    name: &N,
) -> std::fmt::Result {
    f.write_char(' ')?;
    name.render(f)
}

#[inline]
fn render_attr<N: AttributeName, V: AttributeValue>(
    f: &mut std::fmt::Formatter<'_>,
    name: &N,
    value: &V,
) -> std::fmt::Result {
    render_attr_name_only(f, name)?;
    f.write_char('=')?;
    f.write_char('"')?;
    value.render(f)?;
    f.write_char('"')
}

/// Wrapper used for displaying attributes in elements
///
/// This wrapper can print attributes with or without values.
/// It can also handle attributes wrapped in an `Option` and will behave accordingly.
///
/// # Examples
///
/// ```rust
/// let html = another_html_builder::Buffer::default()
///     .node("div")
///     .attr("name-only")
///     .attr(("name", "value"))
///     .attr(Some(("other", "value")))
///     .attr(("with-number", 42))
///     .close()
///     .into_inner();
/// assert_eq!(
///     html,
///     "<div name-only name=\"value\" other=\"value\" with-number=\"42\" />"
/// );
/// ```
///
/// # Extending
///
/// It's possible to implement attributes with custom types, just by implementing the [AttributeName] and [AttributeValue] traits.
///
/// ```rust
/// use std::fmt::{Display, Write};
///
/// struct ClassNames<'a>(&'a [&'static str]);
///
/// impl<'a> another_html_builder::attribute::AttributeValue for ClassNames<'a> {
///     fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         for (index, inner) in self.0.iter().enumerate() {
///             if (index > 0) {
///                 f.write_char(' ')?;
///             }
///             // this could be avoided if you consider it is escaped by default
///             another_html_builder::attribute::EscapedValue(inner).fmt(f)?;
///         }
///         Ok(())
///     }
/// }
///
/// let html = another_html_builder::Buffer::default()
///     .node("div")
///     .attr(("class", ClassNames(&["foo", "bar"])))
///     .close()
///     .into_inner();
/// assert_eq!(html, "<div class=\"foo bar\" />");
/// ```
pub struct Attribute<T>(pub T);

impl<N: AttributeName> std::fmt::Display for Attribute<Option<N>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref inner) = self.0 {
            render_attr_name_only(f, inner)
        } else {
            Ok(())
        }
    }
}

impl<N: AttributeName> std::fmt::Display for Attribute<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        render_attr_name_only(f, &self.0)
    }
}

impl<N: AttributeName, V: AttributeValue> std::fmt::Display for Attribute<Option<(N, V)>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((name, value)) = &self.0 {
            render_attr(f, name, value)
        } else {
            Ok(())
        }
    }
}

impl<N: AttributeName, V: AttributeValue> std::fmt::Display for Attribute<(N, V)> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, value) = &self.0;
        render_attr(f, name, value)
    }
}

attribute_value!(bool);
attribute_value!(u8);
attribute_value!(u16);
attribute_value!(u32);
attribute_value!(u64);
attribute_value!(usize);
attribute_value!(i8);
attribute_value!(i16);
attribute_value!(i32);
attribute_value!(i64);
attribute_value!(isize);
