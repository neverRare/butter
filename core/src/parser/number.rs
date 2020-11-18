fn parse_digit(radix: u64, ch: char) -> u64 {
    let digit = match ch {
        '0'..='9' => ch as u64 - '0' as u64,
        'a'..='z' => ch as u64 + 10 - 'a' as u64,
        'A'..='Z' => ch as u64 + 10 - 'A' as u64,
        _ => panic!(),
    };
    assert!(digit < radix);
    digit
}
pub fn parse_number(radix: u64, src: &str) -> Option<u64> {
    (0..)
        .map(|i| radix.checked_pow(i))
        .zip(
            src.chars()
                .rev()
                .filter(|ch| *ch != '_')
                .map(|ch| parse_digit(radix, ch)),
        )
        .map(|(place, digit)| place.and_then(|place| place.checked_mul(digit)))
        .fold(Some(0), |left, right| match (left, right) {
            (Some(left), Some(right)) => left.checked_add(right),
            _ => None,
        })
}
#[cfg(test)]
mod test {
    use crate::parser::number::parse_number;

    #[test]
    fn parse() {
        assert_eq!(parse_number(16, "18e"), Some(398));
    }
}
