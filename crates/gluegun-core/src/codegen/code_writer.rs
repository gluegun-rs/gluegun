use std::io::Write;

/// A `CodeWriter` can be used with the [`std::fmt::write`][] macro to generate indented code.
/// 
/// Example:
/// 
/// ```rust
/// let cw = CodeWriter::new(...);
/// write!(cw, "void foo {");
/// write!(cw, "this will be indented")
/// write!(cw, "}");
/// ```
/// 
/// would generate
/// 
/// ```
/// void foo() {
///     this will be indented
/// }
/// ```
pub struct CodeWriter<'w> {
    writer: Box<dyn Write + 'w>,
    indent: usize,
}

impl<'w> CodeWriter<'w> {
    pub(crate) fn new(writer: impl Write + 'w) -> Self {
        Self { writer: Box::new(writer), indent: 0 }
    }
    
    pub fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> anyhow::Result<()> {
        let mut string = String::new();
        std::fmt::write(&mut string, fmt).unwrap();

        if string.starts_with("}") || string.starts_with(")") || string.starts_with("]") {
            self.indent -= 1;
        }

        write!(
            self.writer,
            "{:indent$}{}\n",
            "",
            string,
            indent = self.indent * 4
        )?;

        if string.ends_with("{") || string.ends_with("(") || string.ends_with("[") {
            self.indent += 1;
        }

        Ok(())
    }
}
