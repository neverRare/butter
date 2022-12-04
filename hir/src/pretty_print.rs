use std::{
    convert::Infallible,
    io::{self, Write},
    iter::{once, repeat},
};

pub trait PrettyPrint {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree>;
    fn pretty_print(
        &self,
        writer: &mut dyn Write,
        indent: &'static str,
        max: usize,
    ) -> io::Result<()> {
        self.to_pretty_print().write(writer, indent, max)
    }
}
pub trait PrettyPrintTree {
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
    level: usize,
    indent: &'static str,
    max: usize,
    newline: bool,
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
impl<T: PrettyPrintTree + ?Sized> PrettyPrintTree for Box<T> {
    fn write_len(&self) -> Option<usize> {
        <T as PrettyPrintTree>::write_len(self)
    }
    fn write_line(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()> {
        <T as PrettyPrintTree>::write_line(self, writer, state)?;
        Ok(())
    }
    fn write_multiline(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()> {
        <T as PrettyPrintTree>::write_multiline(self, writer, state)?;
        Ok(())
    }
}
impl PrettyPrintTree for Infallible {
    fn write_len(&self) -> Option<usize> {
        unreachable!();
    }
    fn write_line(&self, _: &mut dyn Write, _: PrettyPrintState) -> io::Result<()> {
        unreachable!();
    }
    fn write_multiline(&self, _: &mut dyn Write, _: PrettyPrintState) -> io::Result<()> {
        unreachable!();
    }
}
impl PrettyPrintTree for str {
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
impl PrettyPrintTree for String {
    fn write_len(&self) -> Option<usize> {
        Some(self.len())
    }
    fn write_line(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()> {
        <str as PrettyPrintTree>::write_line(self, writer, state)?;
        Ok(())
    }
    fn write_multiline(&self, writer: &mut dyn Write, state: PrettyPrintState) -> io::Result<()> {
        self.write_line(writer, state)?;
        Ok(())
    }
}
struct Indent(Box<dyn PrettyPrintTree>);
impl PrettyPrintTree for Indent {
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
struct Sequence<T> {
    content: T,
    multiline_override: Option<bool>,
}
impl<T> PrettyPrintTree for Sequence<T>
where
    for<'a> &'a T: IntoIterator<Item = &'a Box<dyn PrettyPrintTree>>,
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
pub fn indent(content: Box<dyn PrettyPrintTree>) -> Box<dyn PrettyPrintTree> {
    Box::new(Indent(content))
}
fn array_sequence<const L: usize>(
    content: [Box<dyn PrettyPrintTree>; L],
    multiline_override: Option<bool>,
) -> Box<dyn PrettyPrintTree> {
    Box::new(Sequence {
        content,
        multiline_override,
    })
}
pub fn bracket(
    open: &str,
    close: &str,
    content: Box<dyn PrettyPrintTree>,
) -> Box<dyn PrettyPrintTree> {
    array_sequence(
        [
            Box::new(open.to_string()),
            indent(content),
            Box::new(close.to_string()),
        ],
        None,
    )
}
pub fn line<const L: usize>(content: [Box<dyn PrettyPrintTree>; L]) -> Box<dyn PrettyPrintTree> {
    Box::new(Sequence {
        content,
        multiline_override: Some(false),
    })
}
pub fn prefix(prefix: &str, content: Box<dyn PrettyPrintTree>) -> Box<dyn PrettyPrintTree> {
    line([Box::new(prefix.to_string()), indent(content)])
}
pub fn postfix(postfix: &str, content: Box<dyn PrettyPrintTree>) -> Box<dyn PrettyPrintTree> {
    line([indent(content), Box::new(postfix.to_string())])
}
pub fn sequence(
    content: impl IntoIterator<Item = Box<dyn PrettyPrintTree>>,
) -> Box<dyn PrettyPrintTree> {
    Box::new(Sequence {
        content: content.into_iter().collect::<Vec<_>>(),
        multiline_override: None,
    })
}
pub fn multiline_sequence(
    content: impl IntoIterator<Item = Box<dyn PrettyPrintTree>>,
) -> Box<dyn PrettyPrintTree> {
    Box::new(Sequence {
        content: content.into_iter().collect::<Vec<_>>(),
        multiline_override: Some(true),
    })
}
#[cfg(test)]
mod test {
    use super::PrettyPrintTree;
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
                indent(array_sequence(
                    [Box::new("a".to_string()), Box::new("b".to_string())],
                    Some(true),
                )),
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
                array_sequence(
                    [
                        Box::new("(".to_string()),
                        indent(Box::new("b".to_string())),
                        Box::new(")".to_string()),
                    ],
                    Some(true),
                ),
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
                indent(Box::new("a".to_string())),
                indent(Box::new("b".to_string())),
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
                indent(Box::new("a".to_string())),
                indent(Box::new("b".to_string())),
            ],
            Some(true),
        );
        let mut s = <Vec<u8>>::new();
        tree.write(&mut s, "    ", 80).unwrap();
        assert_eq!(String::from_utf8_lossy(&s), expected);
    }
}
