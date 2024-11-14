# Another HTML builder

The goal of this builder is to be simple, to only rely on the standard library, to avoid copying values and write directly to a buffer.
There is no lock involved, the ownership of the buffer is the only thing that will avoid race conditions.

## Example

```rust
const STYLE_PATH: &str = "/assets/style-0.0.1.css";

// you can define components like this
fn render_head(buf: Buffer<String, Body<'_>>) -> Buffer<String, Body<'_>> {
    buf.node("head").content(|buf| {
        buf.node("meta")
            .attr(("charset", "utf-8"))
            .close()
            .node("meta")
            .attr(("name", "viewport"))
            .attr(("content", "width=device-width, initial-scale=1"))
            .close()
            .node("title")
            .content(|buf| buf.text("This is a wonderful title"))
            .node("link")
            .attr(("rel", "stylesheet"))
            .attr(("href", STYLE_PATH))
            .close()
    })
}

let html = another_html_builder::Buffer::default()
    .doctype()
    .node("html")
    .attr(("lang", "en"))
    .content(|buf| {
        let buf = render_head(buf);
        buf.node("body").content(|buf| {
            buf.node("div")
                .attr(("class", "card"))
                .content(|buf| {
                    buf.node("div")
                        .attr(("class", "card-header text-center")) // with string attributes
                        .attr(("attr-index", 42)) // with a number
                        .attr(("attr-visible", true)) // with a boolean
                        .content(|buf| buf.text("Show me what you got!"))
                        .node("div")
                        .attr(("class", "card-body"))
                        .content(|buf| buf.text("This will be encoded < and this will be escaped \""))
                })
        })
    })
    .into_inner();
```
