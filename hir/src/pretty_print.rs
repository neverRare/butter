use std::{
    io::{self, Write},
    iter::{once, repeat},
};

pub trait PrettyPrint {
    fn write_len(&self) -> Option<usize>;
    fn write_line(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()>;
    fn write_multiline(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()>;
    fn pretty_write(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()> {
        let fit = match self.write_len() {
            Some(len) => state.level * state.indent.len() + len <= state.max,
            None => false,
        };
        if fit {
            self.write_line(writer, state)?;
        } else {
            self.write_multiline(writer, state)?;
        }
        Ok(())
    }
    fn write(&self, writer: &mut dyn Write, indent: &'static str, max: usize) -> io::Result<()> {
        self.pretty_write(
            writer,
            PrettyPrintState {
                level: 0,
                indent,
                max,
                newline: true,
            },
        )?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PrettyPrintState {
    pub level: usize,
    pub indent: &'static str,
    pub max: usize,
    pub newline: bool,
}
impl PrettyPrintState {
    fn incr(&mut self) {
        self.level += 1;
    }
    fn write_indent(&self, writer: &mut impl Write) -> io::Result<()> {
        for _ in 0..self.level {
            write!(writer, "{}", self.indent)?;
        }
        Ok(())
    }
}
impl<T: PrettyPrint + ?Sized> PrettyPrint for Box<T> {
    fn write_len(&self) -> Option<usize> {
        <T as PrettyPrint>::write_len(self)
    }
    fn write_line(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()> {
        <T as PrettyPrint>::write_line(self, writer, state)?;
        Ok(())
    }
    fn write_multiline(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()> {
        <T as PrettyPrint>::write_multiline(self, writer, state)?;
        Ok(())
    }
}
impl PrettyPrint for str {
    fn write_len(&self) -> Option<usize> {
        Some(self.len())
    }
    fn write_line(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()> {
        let s = if state.newline {
            self.trim_start()
        } else {
            self
        };
        write!(writer, "{}", s)?;
        Ok(())
    }
    fn write_multiline(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()> {
        self.write_line(writer, state)?;
        Ok(())
    }
}
impl PrettyPrint for String {
    fn write_len(&self) -> Option<usize> {
        Some(self.len())
    }
    fn write_line(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()> {
        <str as PrettyPrint>::write_line(self, writer, state)?;
        Ok(())
    }
    fn write_multiline(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()> {
        self.write_line(writer, state)?;
        Ok(())
    }
}
pub struct Indent(pub Box<dyn PrettyPrint>);
impl PrettyPrint for Indent {
    fn write_len(&self) -> Option<usize> {
        self.0.write_len()
    }
    fn write_line(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()> {
        let mut state = state;
        if state.newline {
            write!(writer, "{}", state.indent)?;
            state.incr();
        };
        self.0.write_line(writer, state)?;
        Ok(())
    }
    fn write_multiline(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()> {
        let mut state = state;
        if state.newline {
            write!(writer, "{}", state.indent)?;
            state.incr();
        };
        self.0.pretty_write(writer, state)?;
        Ok(())
    }
}
pub struct Sequence<T> {
    pub content: T,
    pub multiline_override: Option<bool>,
}
impl<T> PrettyPrint for Sequence<T>
where
    for<'a> &'a T: IntoIterator<Item = &'a Box<dyn PrettyPrint>>,
{
    fn write_len(&self) -> Option<usize> {
        if self.multiline_override == Some(true) {
            None
        } else {
            Some(
                self.content
                    .into_iter()
                    .map(|a| a.write_len())
                    .try_fold(0, |a, b| Some(a + b?))?,
            )
        }
    }
    fn write_line(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()> {
        let iter = self
            .content
            .into_iter()
            .zip(once(false).chain(repeat(true)));
        for (tree, rest) in iter {
            let state = PrettyPrintState {
                newline: state.newline && !rest,
                ..state
            };
            tree.write_line(writer, state)?;
        }
        Ok(())
    }
    fn write_multiline(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()> {
        let mut writer = writer;
        let iter = self
            .content
            .into_iter()
            .zip(once(false).chain(repeat(true)));
        if self.multiline_override == Some(false) {
            for (tree, rest) in iter {
                let state = PrettyPrintState {
                    newline: state.newline && !rest,
                    ..state
                };
                tree.pretty_write(writer, state)?;
            }
        } else {
            for (tree, rest) in iter {
                if rest {
                    writeln!(writer)?;
                    state.write_indent(&mut writer)?;
                }
                let state = PrettyPrintState {
                    newline: state.newline || rest,
                    ..state
                };
                tree.pretty_write(writer, state)?;
            }
        }
        Ok(())
    }
}
pub type ArraySequence<const L: usize> = Sequence<[Box<dyn PrettyPrint>; L]>;
pub fn indent(content: impl PrettyPrint + 'static) -> Indent {
    Indent(Box::new(content))
}
pub fn array_sequence<const L: usize>(
    content: [Box<dyn PrettyPrint>; L],
    multiline_override: Option<bool>,
) -> ArraySequence<L> {
    Sequence {
        content,
        multiline_override,
    }
}
pub fn singleline_sequence<const L: usize>(content: [Box<dyn PrettyPrint>; L]) -> ArraySequence<L> {
    Sequence {
        content,
        multiline_override: Some(false),
    }
}
pub fn bracket(open: &str, close: &str, content: impl PrettyPrint + 'static) -> ArraySequence<3> {
    array_sequence(
        [
            Box::new(open.to_string()),
            Box::new(indent(content)),
            Box::new(close.to_string()),
        ],
        None,
    )
}
#[cfg(test)]
mod test {
    use super::PrettyPrint;
    use crate::pretty_print::{array_sequence, indent};

    #[test]
    fn nested() {
        let expected = "\
(
    a
    b
)";
        let tree = array_sequence(
            [
                Box::new("(".to_string()),
                Box::new(indent(array_sequence(
                    [Box::new("a".to_string()), Box::new("b".to_string())],
                    Some(true),
                ))),
                Box::new(")".to_string()),
            ],
            None,
        );
        let mut s = <Vec<u8>>::new();
        tree.write(&mut s, "    ", 80).unwrap();
        assert_eq!(String::from_utf8_lossy(&s), expected);
    }
    #[test]
    fn keep_single_line() {
        let expected = "\
a(
    b
)";
        let tree = array_sequence(
            [
                Box::new("a".to_string()),
                Box::new(array_sequence(
                    [
                        Box::new("(".to_string()),
                        Box::new(indent("b".to_string())),
                        Box::new(")".to_string()),
                    ],
                    Some(true),
                )),
            ],
            Some(false),
        );
        let mut s = <Vec<u8>>::new();
        tree.write(&mut s, "    ", 80).unwrap();
        assert_eq!(String::from_utf8_lossy(&s), expected);
    }
    #[test]
    fn indent_first() {
        let expected = "    ab";
        let tree = array_sequence(
            [
                Box::new(indent("a".to_string())),
                Box::new(indent("b".to_string())),
            ],
            Some(false),
        );
        let mut s = <Vec<u8>>::new();
        tree.write(&mut s, "    ", 80).unwrap();
        assert_eq!(String::from_utf8_lossy(&s), expected);
    }
    #[test]
    fn indent_on_newline() {
        let expected = "    a\n    b";
        let tree = array_sequence(
            [
                Box::new(indent("a".to_string())),
                Box::new(indent("b".to_string())),
            ],
            Some(true),
        );
        let mut s = <Vec<u8>>::new();
        tree.write(&mut s, "    ", 80).unwrap();
        assert_eq!(String::from_utf8_lossy(&s), expected);
    }
}
