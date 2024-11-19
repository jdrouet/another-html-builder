use std::fmt::Write;

fn write_escaped_attrobute_str<W: Write>(f: &mut W, value: &str) -> std::fmt::Result {
    for c in value.chars() {
        match c {
            '"' => f.write_str("\\\"")?,
            other => f.write_char(other)?,
        }
    }
    Ok(())
}

fn write_escaped_content_str<W: Write>(f: &mut W, value: &str) -> std::fmt::Result {
    for c in value.chars() {
        match c {
            '&' => f.write_str("&amp;")?,
            '<' => f.write_str("&lt;")?,
            '>' => f.write_str("&gt;")?,
            '"' => f.write_str("&quot;")?,
            '\'' => f.write_str("&#x27;")?,
            '/' => f.write_str("&#x2F;")?,
            other => f.write_char(other)?,
        }
    }
    Ok(())
}

macro_rules! attribute_value {
    ($type:ty) => {
        impl AttributeValue for $type {
            fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "\"{self}\"")
            }
        }
    };
}

pub trait AttributeName {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl AttributeName for &str {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self)
    }
}

pub trait AttributeValue {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl AttributeValue for &str {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('"')?;
        write_escaped_attrobute_str(f, self)?;
        f.write_char('"')
    }
}

fn render_attr_name_only<N: AttributeName>(
    f: &mut std::fmt::Formatter<'_>,
    name: &N,
) -> std::fmt::Result {
    f.write_char(' ')?;
    name.render(f)
}

fn render_attr<N: AttributeName, V: AttributeValue>(
    f: &mut std::fmt::Formatter<'_>,
    name: &N,
    value: &V,
) -> std::fmt::Result {
    render_attr_name_only(f, name)?;
    f.write_char('=')?;
    value.render(f)
}

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

#[derive(Debug)]
pub enum Body<'a> {
    Root,
    Element {
        name: &'a str,
        parent: Box<Body<'a>>,
    },
}

impl Body<'_> {
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

#[derive(Debug)]
pub struct Element<'a> {
    parent: Body<'a>,
    name: &'a str,
}

#[derive(Clone, Debug)]
pub struct Buffer<W, C> {
    inner: W,
    current: C,
}

impl Default for Buffer<String, Body<'static>> {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer<String, Body<'static>> {
    pub fn new() -> Self {
        Self {
            inner: String::default(),
            current: Body::Root,
        }
    }
}

impl<W> Buffer<W, Body<'_>> {
    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl Buffer<String, Body<'_>> {
    pub fn inner(&self) -> &str {
        self.inner.as_str()
    }
}

impl<W: std::fmt::Write> Buffer<W, Body<'_>> {
    pub fn doctype(mut self) -> Self {
        self.inner.write_str("<!DOCTYPE html>").unwrap();
        self
    }

    pub fn try_doctype(mut self) -> Result<Self, std::fmt::Error> {
        self.inner.write_str("<!DOCTYPE html>")?;
        Ok(self)
    }
}

impl<'a, W: std::fmt::Write> Buffer<W, Body<'a>> {
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

    pub fn try_cond<F>(
        self,
        condition: bool,
        children: F,
    ) -> Result<Buffer<W, Body<'a>>, std::fmt::Error>
    where
        F: FnOnce(Buffer<W, Body>) -> Result<Buffer<W, Body>, std::fmt::Error>,
    {
        if condition {
            children(self)
        } else {
            Ok(self)
        }
    }

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
    ) -> Result<Buffer<W, Body<'a>>, std::fmt::Error>
    where
        F: FnOnce(Buffer<W, Body>, V) -> Result<Buffer<W, Body>, std::fmt::Error>,
    {
        if let Some(inner) = value {
            children(self, inner)
        } else {
            Ok(self)
        }
    }

    pub fn node(mut self, tag: &'a str) -> Buffer<W, Element<'a>> {
        write!(&mut self.inner, "<{tag}").unwrap();
        Buffer {
            inner: self.inner,
            current: Element {
                name: tag,
                parent: self.current,
            },
        }
    }

    pub fn try_node(mut self, tag: &'a str) -> Result<Buffer<W, Element<'a>>, std::fmt::Error> {
        write!(&mut self.inner, "<{tag}")?;
        Ok(Buffer {
            inner: self.inner,
            current: Element {
                name: tag,
                parent: self.current,
            },
        })
    }

    pub fn raw<V: std::fmt::Display>(mut self, value: V) -> Self {
        write!(&mut self.inner, "{value}").unwrap();
        self
    }

    pub fn try_raw<V: std::fmt::Display>(mut self, value: V) -> Result<Self, std::fmt::Error> {
        write!(&mut self.inner, "{value}")?;
        Ok(self)
    }

    pub fn text(mut self, content: &str) -> Self {
        write_escaped_content_str(&mut self.inner, content).unwrap();
        self
    }

    pub fn try_text(mut self, content: &str) -> Result<Self, std::fmt::Error> {
        write_escaped_content_str(&mut self.inner, content)?;
        Ok(self)
    }
}

impl<'a, W: std::fmt::Write> Buffer<W, Element<'a>> {
    pub fn attr<T>(mut self, attr: T) -> Self
    where
        Attribute<T>: std::fmt::Display,
    {
        write!(&mut self.inner, "{}", Attribute(attr)).unwrap();
        self
    }

    #[inline]
    pub fn cond_attr<T>(self, condition: bool, attr: T) -> Self
    where
        Attribute<T>: std::fmt::Display,
    {
        if condition {
            self.attr(attr)
        } else {
            self
        }
    }

    #[inline]
    pub fn try_attr<T>(mut self, attr: T) -> Result<Self, std::fmt::Error>
    where
        Attribute<T>: std::fmt::Display,
    {
        write!(&mut self.inner, "{}", Attribute(attr))?;
        Ok(self)
    }

    #[inline]
    pub fn try_cond_attr<T>(self, condition: bool, attr: T) -> Result<Self, std::fmt::Error>
    where
        Attribute<T>: std::fmt::Display,
    {
        if condition {
            self.try_attr(attr)
        } else {
            Ok(self)
        }
    }

    pub fn close(mut self) -> Buffer<W, Body<'a>> {
        self.inner.write_str(" />").unwrap();
        Buffer {
            inner: self.inner,
            current: self.current.parent,
        }
    }

    pub fn try_close(mut self) -> Result<Buffer<W, Body<'a>>, std::fmt::Error> {
        self.inner.write_str(" />")?;
        Ok(Buffer {
            inner: self.inner,
            current: self.current.parent,
        })
    }

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

    pub fn try_content<F>(mut self, children: F) -> Result<Buffer<W, Body<'a>>, std::fmt::Error>
    where
        F: FnOnce(Buffer<W, Body>) -> Result<Buffer<W, Body>, std::fmt::Error>,
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
    use super::*;

    #[test]
    fn should_rollback_after_content() {
        let buffer = Buffer::new().node("a").content(|buf| buf);
        assert!(
            matches!(buffer.current, Body::Root),
            "found {:?}",
            buffer.current
        );
    }

    #[test]
    fn simple_html() {
        let html = Buffer::new()
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
        let html = Buffer::new()
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
        let html = Buffer::new()
            .node("p")
            .content(|b| b.text("asd\"weiofew!/<>"))
            .into_inner();
        assert_eq!(html, "<p>asd&quot;weiofew!&#x2F;&lt;&gt;</p>");
    }

    #[test]
    fn with_optional_attributes() {
        let html = Buffer::new()
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
    fn with_conditional_attributes() {
        let html = Buffer::new()
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
        let html = Buffer::new()
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
        let html = Buffer::new()
            .node("div")
            .content(|buf| buf.optional(error, |buf, msg| buf.text(msg)))
            .into_inner();
        assert_eq!(html, "<div>This is an error</div>");
    }
}
