//! Set of extension implementations allowing to write to [std::fmt::Write] or [std::io::Write].

/// Abstraction layer allowing not only to write to [std::fmt::Write] but also to [std::io::Write].
pub trait WriterExt {
    type Error: std::error::Error;

    /// Function that will write any element that implement [std::fmt::Display].
    fn write<E: std::fmt::Display>(&mut self, input: E) -> Result<(), Self::Error>;
    fn write_str(&mut self, input: &str) -> Result<(), Self::Error>;
    fn write_char(&mut self, input: char) -> Result<(), Self::Error>;
}

/// Wrapper for writer implementing [std::fmt::Write].
pub struct FmtWriter<W>(pub W);

impl<W: std::fmt::Write> WriterExt for FmtWriter<W> {
    type Error = std::fmt::Error;
    fn write<E: std::fmt::Display>(&mut self, input: E) -> Result<(), Self::Error> {
        write!(&mut self.0, "{input}")
    }

    fn write_str(&mut self, input: &str) -> Result<(), Self::Error> {
        self.0.write_str(input)
    }

    fn write_char(&mut self, input: char) -> Result<(), Self::Error> {
        self.0.write_char(input)
    }
}

/// Wrapper for writer implementing [std::io::Write].
pub struct IoWriter<W>(pub W);

impl<W: std::io::Write> WriterExt for IoWriter<W> {
    type Error = std::io::Error;

    fn write<E: std::fmt::Display>(&mut self, input: E) -> std::io::Result<()> {
        write!(self.0, "{input}")
    }

    fn write_str(&mut self, input: &str) -> std::io::Result<()> {
        write!(self.0, "{input}")
    }

    fn write_char(&mut self, input: char) -> std::io::Result<()> {
        write!(self.0, "{input}")
    }
}
