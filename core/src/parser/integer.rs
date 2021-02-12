macro_rules! gen_parser {
    ($name:ident, $type:ty $(,)?) => {
        pub fn $name(radix: $type, src: &[u8]) -> Option<$type> {
            (0..)
                .map(|i| radix.checked_pow(i))
                .zip(
                    src.iter()
                        .filter(|ch| **ch != b'_')
                        .rev()
                        .map(|ch| match ch {
                            b'0'..=b'9' => *ch as $type - b'0' as $type,
                            b'a'..=b'z' => *ch as $type + 10 - b'a' as $type,
                            b'A'..=b'Z' => *ch as $type + 10 - b'A' as $type,
                            _ => panic!(),
                        }),
                )
                .map(|(place, digit)| place.and_then(|place| place.checked_mul(digit)))
                .fold(Some(0), |left, right| match (left, right) {
                    (Some(left), Some(right)) => left.checked_add(right),
                    _ => None,
                })
        }
    };
}
gen_parser!(parse_u64, u64);
gen_parser!(parse_i32, i32);
#[cfg(test)]
mod test {
    use crate::parser::integer::parse_u64;

    #[test]
    fn parse() {
        assert_eq!(parse_u64(16, b"18e"), Some(398));
    }
}
