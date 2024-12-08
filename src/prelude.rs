pub struct FmtWriter<W>(pub W);
pub struct IoWriter<W>(pub W);

pub trait WriterExt {
    fn write<E: std::fmt::Display>(&mut self, input: E) -> std::io::Result<()>;
    fn write_str(&mut self, input: &str) -> std::io::Result<()>;
    fn write_char(&mut self, input: char) -> std::io::Result<()>;
}

impl<W: std::fmt::Write> WriterExt for FmtWriter<W> {
    fn write<E: std::fmt::Display>(&mut self, input: E) -> std::io::Result<()> {
        write!(&mut self.0, "{input}").map_err(std::io::Error::other)
    }

    fn write_str(&mut self, input: &str) -> std::io::Result<()> {
        self.0.write_str(input).map_err(std::io::Error::other)
    }

    fn write_char(&mut self, input: char) -> std::io::Result<()> {
        self.0.write_char(input).map_err(std::io::Error::other)
    }
}

impl<W: std::io::Write> WriterExt for IoWriter<W> {
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
