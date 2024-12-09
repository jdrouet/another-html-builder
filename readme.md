# Another HTML builder

The goal of this builder is to be simple, to only rely on the standard library, to avoid copying values and write directly to a buffer.
There is no lock involved, the ownership of the buffer is the only thing that will avoid race conditions.

## Example

```rust
use another_html_builder::attribute::AttributeValue;
use another_html_builder::prelude::WriterExt;
use another_html_builder::{Body, Buffer};

// define your own custom kind of attributes
enum Lang {
    En,
    Fr,
}

impl AttributeValue for Lang {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::En => "en",
            Self::Fr => "fr",
        })
    }
}

// create custom components
struct Head {
    title: &'static str,
}

impl Default for Head {
    fn default() -> Self {
        Self {
            title: "Hello world!",
        }
    }
}

impl Head {
    fn render<'a, W: WriterExt>(&self, buf: Buffer<W, Body<'a>>) -> Buffer<W, Body<'a>> {
        buf.node("head")
            .content(|buf| buf.node("title").content(|buf| buf.text(self.title)))
    }
}

let head = Head::default();
let html = Buffer::default()
    .doctype()
    .node("html")
    .attr(("lang", Lang::Fr))
    .content(|buf| head.render(buf))
    .into_inner();
assert_eq!(
    html,
    "<!DOCTYPE html><html lang=\"fr\"><head><title>Hello world!</title></head></html>"
);
```
