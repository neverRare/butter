fn span_pos_checked<'a>(src: &'a str, span: &'a str) -> Option<usize> {
    let src_pos = src.as_ptr() as usize;
    let pos = (span.as_ptr() as usize).checked_sub(src_pos)?;
    if pos <= src.len() {
        Some(pos)
    } else {
        None
    }
}
fn span_pos<'a>(src: &'a str, span: &'a str) -> usize {
    match span_pos_checked(src, span) {
        Some(pos) => pos,
        None => panic!("{:?} does not lie inside {:?}", span, src),
    }
}
pub fn span_from_spans<'a>(src: &'a str, left: &'a str, right: &'a str) -> &'a str {
    let left = span_pos(src, left);
    let right = span_pos(src, right) + right.len();
    &src[left..right]
}
