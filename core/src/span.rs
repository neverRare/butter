use std::fmt::{Display, Error, Formatter};

pub trait ExplainSpan {
    fn explain(&self) -> (&str, Option<&str>);
}
#[derive(Debug, PartialEq, Eq)]
pub struct Span<'a, T> {
    src: &'a str,
    line_indices: Vec<usize>,
    pub note: T,
    from: usize,
    to: usize,
}
impl<'a, T> Span<'a, T> {
    pub fn new(src: &'a str, note: T, from: usize, to: usize) -> Self {
        Self {
            src,
            line_indices: src.match_indices('\n').map(|(ind, _)| ind + 1).collect(),
            note,
            from,
            to,
        }
    }
    pub fn fit_from(self, src: &'a str) -> Self {
        let inside = self.src.as_ptr() as usize;
        let outside = src.as_ptr() as usize;
        assert!(inside >= outside && inside + self.src.len() <= outside + src.len());
        let delta = inside - outside;
        Self::new(src, self.note, delta + self.from, delta + self.to)
    }
    pub fn map<F, U>(self, mapper: F) -> Span<'a, U>
    where
        F: FnOnce(T) -> U,
    {
        Span {
            src: self.src,
            line_indices: self.line_indices,
            note: mapper(self.note),
            from: self.from,
            to: self.to,
        }
    }
    fn is_multiline(&self) -> bool {
        self.line_indices
            .iter()
            .any(|ind| self.from < *ind && self.to > *ind)
    }
    fn line_no(&self, i: usize) -> usize {
        assert!(i < self.src.len());
        self.line_indices
            .iter()
            .enumerate()
            .find_map(|(line, ind)| if ind > &i { Some(line) } else { None })
            .unwrap_or_else(|| self.line_indices.len())
    }
    fn line_offset(&self, i: usize) -> usize {
        assert!(i < self.src.len());
        let line_ind = self.line_indices[self.line_no(i) - 1];
        i - line_ind
    }
    fn line_str(&self, line_no: usize) -> &str {
        let from = self.line_indices[line_no - 1];
        if line_no < self.line_indices.len() {
            &self.src[from..self.line_indices[line_no] - 1]
        } else {
            &self.src[from..]
        }
    }
    fn max_line_no(&self) -> usize {
        self.line_indices.len()
    }
}
impl<'a, T> Display for Span<'a, T>
where
    T: ExplainSpan,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let (description, note) = self.note.explain();
        writeln!(f, "{}", description);
        if self.is_multiline() {
            let line_no_from = self.line_no(self.from);
            let line_no_to = self.line_no(self.to);
            let start_line = self.line_str(line_no_from);
            let (before_start, after_start) = start_line.split_at(self.line_offset(self.from));
            if !before_start.chars().all(char::is_whitespace) {
                writeln!(
                    f,
                    "{:>3}  {}",
                    start_line,
                    before_start,
                );
            }
            writeln!(
                f,
                "{:>3}/ {}{}",
                start_line,
                " ".repeat(before_start.len()),
                after_start,
            );
            for line_no in line_no_from + 1..line_no_to {
                writeln!(
                    f,
                    "{:>3}| {}",
                    line_no,
                    self.line_str(line_no),
                );
            }
            let end_line = self.line_str(line_no_to);
            let (before_end, after_end) = end_line.split_at(self.line_offset(self.to));
            writeln!(
                f,
                "{:>3}\\ {}",
                end_line,
                before_end,
            );
            if !after_end.chars().all(char::is_whitespace) {
                writeln!(
                    f,
                    "{:>3}  {}{}",
                    end_line,
                    " ".repeat(before_end.len()),
                    after_end,
                );
            }
        } else {
            let line_no = self.line_no(self.from);
            if line_no > 1 {
                writeln!(f, "{:>3} {}", line_no - 1, self.line_str(line_no - 1));
            }
            writeln!(f, "{:>3} {}", line_no, self.line_str(line_no));
            writeln!(
                f,
                "    {}{}",
                " ".repeat(self.line_offset(self.from)),
                "-".repeat(self.to - self.from),
            );
            if line_no < self.max_line_no() {
                writeln!(f, "{:>3} {}", line_no + 1, self.line_str(line_no + 1));
            }
        }
        if let Some(note) = note {
            writeln!(f, "Note: {}", note);
        }
        Ok(())
    }
}
pub struct Spans<'a, T, U> {
    summary: T,
    src: &'a str,
    spans: Vec<(U, usize, usize)>,
}
impl<'a, T, U> Spans<'a, T, U> {
    pub fn new(summary: T, src: &'a str, spans: Vec<Span<'a, U>>) -> Self {
        let mut result = vec![];
        for span in spans {
            let Span {
                src: span_src,
                line_indices: _,
                note,
                from,
                to,
            } = span;
            assert_eq!(src, span_src);
            result.push((note, from, to));
        }
        Self {
            summary,
            src,
            spans: result,
        }
    }
}
