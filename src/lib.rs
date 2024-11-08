#![feature(box_into_inner)]

use std::fmt::Write;

macro_rules! attribute_value {
    ($type:ty) => {
        impl AttributeValue for $type {
            fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{self}")
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

pub struct Attribute<T>(pub T);

impl<N: AttributeName> std::fmt::Display for Attribute<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.render(f)
    }
}

impl<N: AttributeName, V: AttributeValue> std::fmt::Display for Attribute<(N, V)> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, value) = &self.0;
        name.render(f)?;
        f.write_char('=')?;
        // TODO escape with string
        f.write_char('"')?;
        value.render(f)?;
        f.write_char('"')
    }
}

attribute_value!(&str);
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

pub trait CanCreateElement {}
pub trait CanAddAttribute {}

#[derive(Debug)]
pub enum Body<'a> {
    Root,
    Element {
        name: &'a str,
        parent: Box<Body<'a>>,
    },
}

impl<'a> Body<'a> {
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
pub struct Buffer<C> {
    inner: String,
    current: C,
}

impl Default for Buffer<Body<'static>> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            current: Body::Root,
        }
    }
}

impl Buffer<Body<'_>> {
    pub fn into_inner(self) -> String {
        self.inner
    }

    pub fn inner(&self) -> &str {
        self.inner.as_str()
    }
}

impl Buffer<Body<'_>> {
    pub fn doctype(mut self) -> Self {
        self.inner.push_str("<!DOCTYPE html>");
        self
    }
}

impl<'a> Buffer<Body<'a>> {
    pub fn start_element(mut self, tag: &'a str) -> Buffer<Element<'a>> {
        self.inner.push('<');
        self.inner.push_str(tag);
        Buffer {
            inner: self.inner,
            current: Element {
                name: tag,
                parent: self.current,
            },
        }
    }

    pub fn text(mut self, content: &str) -> Self {
        self.inner.push_str(content);
        self
    }
}

impl<'a> Buffer<Element<'a>> {
    pub fn attribute<T>(mut self, attr: T) -> Self
    where
        Attribute<T>: std::fmt::Display,
    {
        write!(&mut self.inner, " {}", Attribute(attr)).unwrap();
        self
    }

    pub fn close(mut self) -> Buffer<Body<'a>> {
        self.inner.push_str(" />");
        Buffer {
            inner: self.inner,
            current: self.current.parent,
        }
    }

    pub fn content<F>(mut self, children: F) -> Buffer<Body<'a>>
    where
        F: FnOnce(Buffer<Body>) -> Buffer<Body>,
    {
        self.inner.push('>');
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
                inner.push_str("</");
                inner.push_str(name);
                inner.push('>');
                Buffer {
                    inner,
                    current: Box::into_inner(parent),
                }
            }
            // This should never happen
            Body::Root => Buffer {
                inner,
                current: Body::Root,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_rollback_after_content() {
        let buffer = Buffer::default().start_element("a").content(|buf| buf);
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
            .start_element("html")
            .attribute(("lang", "en"))
            .content(|buf| {
                buf.start_element("head")
                    .content(|buf| {
                        let buf = buf
                            .start_element("meta")
                            .attribute(("charset", "utf-8"))
                            .close();
                        buf.start_element("meta")
                            .attribute(("name", "viewport"))
                            .attribute(("content", "width=device-width, initial-scale=1"))
                            .close()
                    })
                    .start_element("body")
                    .close()
            })
            .into_inner();
        assert_eq!(
            html,
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" /></head><body /></html>"
        );
    }
}
