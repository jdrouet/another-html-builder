//! Just a simple toolkit for writing html.
//!
//! This provides the basic functions needed to write basic html or create components to build a rich and complete UI.
//!
//! # Example
//!
//! In this example, we create a custom attribute and also a custom `Head` element.
//!
//! ```rust
//! use another_html_builder::attribute::AttributeValue;
//! use another_html_builder::prelude::WriterExt;
//! use another_html_builder::{Body, Buffer};
//!
//! enum Lang {
//!     En,
//!     Fr,
//! }
//!
//! impl AttributeValue for Lang {
//!     fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         f.write_str(match self {
//!             Self::En => "en",
//!             Self::Fr => "fr",
//!         })
//!     }
//! }
//!
//! struct Head {
//!     title: &'static str,
//! }
//!
//! impl Default for Head {
//!     fn default() -> Self {
//!         Self {
//!             title: "Hello world!",
//!         }
//!     }
//! }
//!
//! impl Head {
//!     fn render<'a, W: WriterExt>(&self, buf: Buffer<W, Body<'a>>) -> Buffer<W, Body<'a>> {
//!         buf.node("head")
//!             .content(|buf| buf.node("title").content(|buf| buf.text(self.title)))
//!     }
//! }
//!
//! let head = Head::default();
//! let html = Buffer::default()
//!     .doctype()
//!     .node("html")
//!     .attr(("lang", Lang::Fr))
//!     .content(|buf| head.render(buf))
//!     .into_inner();
//! assert_eq!(
//!     html,
//!     "<!DOCTYPE html><html lang=\"fr\"><head><title>Hello world!</title></head></html>"
//! );
//! ```
pub mod attribute;
pub mod content;
pub mod prelude;

use crate::prelude::{FmtWriter, IoWriter, WriterExt};

/// Representation of the inside of an element or the root level.
///
/// This component is made for the [Buffer] to be aware of where it is
/// and provide adequat functions.
#[derive(Debug)]
pub enum Body<'a> {
    /// This represents the root of the DOM. It has not name nor parents.
    Root,
    /// This represents any element with a name.
    Element {
        name: &'a str,
        parent: Box<Body<'a>>,
    },
}

impl Body<'_> {
    /// Generates the path of the current element.
    ///
    /// Note: this will not provid a valide CSS path
    pub fn path(&self) -> String {
        match self {
            Self::Root => String::from("$"),
            Self::Element { name, parent } => {
                let mut parent_path = parent.path();
                parent_path.push_str(" > ");
                parent_path.push_str(name);
                parent_path
            }
        }
    }
}

/// Representation of an element
#[derive(Debug)]
pub struct Element<'a> {
    parent: Body<'a>,
    name: &'a str,
}

/// Wrapper arround a writer element.
#[derive(Clone, Debug)]
pub struct Buffer<W, C> {
    inner: W,
    current: C,
}

impl Default for Buffer<FmtWriter<String>, Body<'static>> {
    fn default() -> Self {
        Self::from(String::new())
    }
}

impl<W: std::fmt::Write> From<W> for Buffer<FmtWriter<W>, Body<'static>> {
    fn from(buffer: W) -> Self {
        Self {
            inner: FmtWriter(buffer),
            current: Body::Root,
        }
    }
}

impl<W: std::io::Write> From<W> for Buffer<IoWriter<W>, Body<'static>> {
    fn from(value: W) -> Self {
        Self {
            inner: IoWriter(value),
            current: Body::Root,
        }
    }
}

impl<W> Buffer<FmtWriter<W>, Body<'_>> {
    pub fn into_inner(self) -> W {
        self.inner.0
    }
}

impl<W> Buffer<IoWriter<W>, Body<'_>> {
    pub fn into_inner(self) -> W {
        self.inner.0
    }
}

impl Buffer<FmtWriter<String>, Body<'_>> {
    pub fn inner(&self) -> &str {
        self.inner.0.as_str()
    }
}

impl<W: WriterExt> Buffer<W, Body<'_>> {
    /// Appends the html doctype to the buffer
    pub fn doctype(mut self) -> Self {
        self.inner.write_str("<!DOCTYPE html>").unwrap();
        self
    }

    /// Tries to append the html doctype to the buffer
    pub fn try_doctype(mut self) -> Result<Self, W::Error> {
        self.inner.write_str("<!DOCTYPE html>")?;
        Ok(self)
    }
}

impl<'a, W: WriterExt> Buffer<W, Body<'a>> {
    /// Conditionally apply some children to an element
    ///
    /// ```rust
    /// let is_error = true;
    /// let html = another_html_builder::Buffer::default()
    ///     .cond(is_error, |buf| {
    ///         buf.node("p").content(|buf| buf.text("ERROR!"))
    ///     })
    ///     .into_inner();
    /// assert_eq!(html, "<p>ERROR!</p>");
    /// ```
    pub fn cond<F>(self, condition: bool, children: F) -> Buffer<W, Body<'a>>
    where
        F: FnOnce(Buffer<W, Body>) -> Buffer<W, Body>,
    {
        if condition {
            children(self)
        } else {
            self
        }
    }

    pub fn try_cond<F>(self, condition: bool, children: F) -> Result<Buffer<W, Body<'a>>, W::Error>
    where
        F: FnOnce(Buffer<W, Body>) -> Result<Buffer<W, Body>, W::Error>,
    {
        if condition {
            children(self)
        } else {
            Ok(self)
        }
    }

    /// Conditionally apply some children to an element depending on an optional
    ///
    /// ```rust
    /// let value: Option<u8> = Some(42);
    /// let html = another_html_builder::Buffer::default()
    ///     .optional(value, |buf, answer| {
    ///         buf.node("p")
    ///             .content(|buf| buf.text("Answer: ").raw(answer))
    ///     })
    ///     .into_inner();
    /// assert_eq!(html, "<p>Answer: 42</p>");
    /// ```
    pub fn optional<V, F>(self, value: Option<V>, children: F) -> Buffer<W, Body<'a>>
    where
        F: FnOnce(Buffer<W, Body>, V) -> Buffer<W, Body>,
    {
        if let Some(inner) = value {
            children(self, inner)
        } else {
            self
        }
    }

    pub fn try_optional<V, F>(
        self,
        value: Option<V>,
        children: F,
    ) -> Result<Buffer<W, Body<'a>>, W::Error>
    where
        F: FnOnce(Buffer<W, Body>, V) -> Result<Buffer<W, Body>, W::Error>,
    {
        if let Some(inner) = value {
            children(self, inner)
        } else {
            Ok(self)
        }
    }

    /// Starts a new node in the buffer
    ///
    /// After calling this function, the buffer will only allow to add attributes,
    /// close the current node or add content to the node.
    ///
    /// ```rust
    /// let html = another_html_builder::Buffer::default()
    ///     .node("p")
    ///     .attr(("foo", "bar"))
    ///     .close()
    ///     .into_inner();
    /// assert_eq!(html, "<p foo=\"bar\" />");
    /// ```
    ///
    /// ```rust
    /// let html = another_html_builder::Buffer::default()
    ///     .node("p")
    ///     .content(|buf| buf.text("hello"))
    ///     .into_inner();
    /// assert_eq!(html, "<p>hello</p>");
    /// ```
    pub fn node(mut self, tag: &'a str) -> Buffer<W, Element<'a>> {
        self.inner.write_char('<').unwrap();
        self.inner.write_str(tag).unwrap();
        Buffer {
            inner: self.inner,
            current: Element {
                name: tag,
                parent: self.current,
            },
        }
    }

    pub fn try_node(mut self, tag: &'a str) -> Result<Buffer<W, Element<'a>>, W::Error> {
        self.inner.write_char('<')?;
        self.inner.write_str(tag)?;
        Ok(Buffer {
            inner: self.inner,
            current: Element {
                name: tag,
                parent: self.current,
            },
        })
    }

    /// Appends some raw content implementing [Display](std::fmt::Display)
    ///
    /// This will not escape the provided value.
    pub fn raw<V: std::fmt::Display>(mut self, value: V) -> Self {
        self.inner.write(value).unwrap();
        self
    }

    pub fn try_raw<V: std::fmt::Display>(mut self, value: V) -> Result<Self, W::Error> {
        self.inner.write(value)?;
        Ok(self)
    }

    /// Appends some text and escape it.
    ///
    /// ```rust
    /// let html = another_html_builder::Buffer::default()
    ///     .node("p")
    ///     .content(|b| b.text("asd\"weiofew!/<>"))
    ///     .into_inner();
    /// assert_eq!(html, "<p>asd&quot;weiofew!&#x2F;&lt;&gt;</p>");
    /// ```
    pub fn text(mut self, input: &str) -> Self {
        self.inner.write(content::EscapedContent(input)).unwrap();
        self
    }

    pub fn try_text(mut self, input: &str) -> Result<Self, W::Error> {
        self.inner.write(content::EscapedContent(input))?;
        Ok(self)
    }
}

impl<'a, W: WriterExt> Buffer<W, Element<'a>> {
    /// Appends an attribute to the current node.
    ///
    /// For more information about how to extend attributes, take a look at the [crate::attribute::Attribute] trait.
    ///
    /// ```rust
    /// let html = another_html_builder::Buffer::default()
    ///     .node("p")
    ///     .attr("single")
    ///     .attr(("hello", "world"))
    ///     .attr(("number", 42))
    ///     .attr(Some(("foo", "bar")))
    ///     .attr(None::<(&str, &str)>)
    ///     .attr(Some("here"))
    ///     .attr(None::<&str>)
    ///     .close()
    ///     .into_inner();
    /// assert_eq!(
    ///     html,
    ///     "<p single hello=\"world\" number=\"42\" foo=\"bar\" here />"
    /// );
    /// ```
    pub fn attr<T>(mut self, attr: T) -> Self
    where
        attribute::Attribute<T>: std::fmt::Display,
    {
        self.inner.write(attribute::Attribute(attr)).unwrap();
        self
    }

    #[inline]
    pub fn try_attr<T>(mut self, attr: T) -> Result<Self, W::Error>
    where
        attribute::Attribute<T>: std::fmt::Display,
    {
        self.inner.write(attribute::Attribute(attr))?;
        Ok(self)
    }

    /// Conditionally appends some attributes
    ///
    /// ```rust
    /// let html = another_html_builder::Buffer::default()
    ///     .node("p")
    ///     .cond_attr(true, ("foo", "bar"))
    ///     .cond_attr(false, ("foo", "baz"))
    ///     .cond_attr(true, "here")
    ///     .cond_attr(false, "not-here")
    ///     .close()
    ///     .into_inner();
    /// assert_eq!(html, "<p foo=\"bar\" here />");
    /// ```
    #[inline]
    pub fn cond_attr<T>(self, condition: bool, attr: T) -> Self
    where
        attribute::Attribute<T>: std::fmt::Display,
    {
        if condition {
            self.attr(attr)
        } else {
            self
        }
    }

    #[inline]
    pub fn try_cond_attr<T>(self, condition: bool, attr: T) -> Result<Self, W::Error>
    where
        attribute::Attribute<T>: std::fmt::Display,
    {
        if condition {
            self.try_attr(attr)
        } else {
            Ok(self)
        }
    }

    /// Closes the current node without providing any content
    ///
    /// ```rust
    /// let html = another_html_builder::Buffer::default()
    ///     .node("p")
    ///     .close()
    ///     .into_inner();
    /// assert_eq!(html, "<p />");
    /// ```
    pub fn close(mut self) -> Buffer<W, Body<'a>> {
        self.inner.write_str(" />").unwrap();
        Buffer {
            inner: self.inner,
            current: self.current.parent,
        }
    }

    pub fn try_close(mut self) -> Result<Buffer<W, Body<'a>>, W::Error> {
        self.inner.write_str(" />")?;
        Ok(Buffer {
            inner: self.inner,
            current: self.current.parent,
        })
    }

    /// Closes the current node and start writing it's content
    ///
    /// When returning the inner callback, the closing element will be written to the buffer
    ///
    /// ```rust
    /// let html = another_html_builder::Buffer::default()
    ///     .node("div")
    ///     .content(|buf| buf.node("p").close())
    ///     .into_inner();
    /// assert_eq!(html, "<div><p /></div>");
    /// ```
    pub fn content<F>(mut self, children: F) -> Buffer<W, Body<'a>>
    where
        F: FnOnce(Buffer<W, Body>) -> Buffer<W, Body>,
    {
        self.inner.write_char('>').unwrap();
        let child_buffer = Buffer {
            inner: self.inner,
            current: Body::Element {
                name: self.current.name,
                parent: Box::new(self.current.parent),
            },
        };
        let Buffer { mut inner, current } = children(child_buffer);
        match current {
            Body::Element { name, parent } => {
                inner.write_str("</").unwrap();
                inner.write_str(name).unwrap();
                inner.write_char('>').unwrap();
                Buffer {
                    inner,
                    current: *parent,
                }
            }
            // This should never happen
            Body::Root => Buffer {
                inner,
                current: Body::Root,
            },
        }
    }

    pub fn try_content<F>(mut self, children: F) -> Result<Buffer<W, Body<'a>>, W::Error>
    where
        F: FnOnce(Buffer<W, Body>) -> Result<Buffer<W, Body>, W::Error>,
    {
        self.inner.write_char('>')?;
        let child_buffer = Buffer {
            inner: self.inner,
            current: Body::Element {
                name: self.current.name,
                parent: Box::new(self.current.parent),
            },
        };
        let Buffer { mut inner, current } = children(child_buffer)?;
        match current {
            Body::Element { name, parent } => {
                inner.write_str("</")?;
                inner.write_str(name)?;
                inner.write_char('>')?;
                Ok(Buffer {
                    inner,
                    current: *parent,
                })
            }
            // This should never happen
            Body::Root => Ok(Buffer {
                inner,
                current: Body::Root,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use super::*;

    #[test]
    fn should_return_inner_value() {
        let buf = Buffer::default().node("a").content(|buf| buf);
        assert_eq!(buf.inner(), "<a></a>");
    }

    #[test]
    fn should_give_node_path() {
        let buf = Buffer::default();
        assert_eq!(buf.current.path(), "$");
        let _buf = buf.node("a").content(|buf| {
            assert_eq!(buf.current.path(), "$ > a");
            buf
        });
    }

    #[test]
    fn should_rollback_after_content() {
        let buffer = Buffer::default().node("a").content(|buf| buf);
        assert!(
            matches!(buffer.current, Body::Root),
            "found {:?}",
            buffer.current
        );
    }

    #[test]
    fn simple_html() {
        let html = Buffer::default()
            .doctype()
            .node("html")
            .attr(("lang", "en"))
            .content(|buf| {
                buf.node("head")
                    .content(|buf| {
                        let buf = buf.node("meta").attr(("charset", "utf-8")).close();
                        buf.node("meta")
                            .attr(("name", "viewport"))
                            .attr(("content", "width=device-width, initial-scale=1"))
                            .close()
                    })
                    .node("body")
                    .close()
            })
            .into_inner();
        assert_eq!(
            html,
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" /></head><body /></html>"
        );
    }

    #[test]
    fn with_special_characters_in_attributes() {
        let html = Buffer::default()
            .node("a")
            .attr(("title", "Let's add a quote \" like this"))
            .attr(("href", "http://example.com?whatever=here"))
            .content(|b| b.text("Click me!"))
            .into_inner();
        assert_eq!(
            html,
            "<a title=\"Let's add a quote \\\" like this\" href=\"http://example.com?whatever=here\">Click me!</a>"
        );
    }

    #[test]
    fn with_special_characters_in_content() {
        let html = Buffer::default()
            .node("p")
            .content(|b| b.text("asd\"weiofew!/<>"))
            .into_inner();
        assert_eq!(html, "<p>asd&quot;weiofew!&#x2F;&lt;&gt;</p>");
    }

    #[test]
    fn with_optional_attributes() {
        let html = Buffer::default()
            .node("p")
            .attr(Some(("foo", "bar")))
            .attr(None::<(&str, &str)>)
            .attr(Some("here"))
            .attr(None::<&str>)
            .close()
            .into_inner();
        assert_eq!(html, "<p foo=\"bar\" here />");
    }

    #[test]
    fn with_attributes() {
        let html = Buffer::default()
            .node("p")
            .attr(("foo", "bar"))
            .attr(("bool", true))
            .attr(("u8", 42u8))
            .attr(("i8", -1i8))
            .close()
            .into_inner();
        assert_eq!(html, "<p foo=\"bar\" bool=\"true\" u8=\"42\" i8=\"-1\" />");
    }

    #[test]
    fn with_conditional_attributes() {
        let html = Buffer::default()
            .node("p")
            .cond_attr(true, ("foo", "bar"))
            .cond_attr(false, ("foo", "baz"))
            .cond_attr(true, "here")
            .cond_attr(false, "not-here")
            .close()
            .into_inner();
        assert_eq!(html, "<p foo=\"bar\" here />");
    }

    #[test]
    fn with_conditional_content() {
        let notification = false;
        let connected = true;
        let html = Buffer::default()
            .node("div")
            .content(|buf| {
                buf.cond(notification, |buf| {
                    buf.node("p")
                        .content(|buf| buf.text("You have a notification"))
                })
                .cond(connected, |buf| buf.text("Welcome!"))
            })
            .into_inner();
        assert_eq!(html, "<div>Welcome!</div>");
    }

    #[test]
    fn with_optional_content() {
        let error = Some("This is an error");
        let html = Buffer::default()
            .node("div")
            .content(|buf| buf.optional(error, |buf, msg| buf.text(msg)))
            .into_inner();
        assert_eq!(html, "<div>This is an error</div>");
    }

    #[test]
    fn should_write_to_io_buffer() {
        let buf = Buffer::from(Cursor::new(Vec::new()));
        let buf = buf.node("div").content(|buf| buf.text("Hello World!"));
        let mut writer = buf.into_inner();
        writer.flush().unwrap();
        let inner = writer.into_inner();
        assert_eq!(&inner, "<div>Hello World!</div>".as_bytes());
    }
}
