#[nutype::nutype(
    sanitize(with = sanitize),
    validate(predicate = validate),
    derive(Serialize, Deserialize)
)]
pub(crate) struct DicomDate(String);

fn sanitize(s: String) -> String {
    if s.contains('/') {
        let mut iter = s.split('/');
        if let Some(month) = iter.next()
            && let Some(day) = iter.next()
            && let Some(year) = iter.next()
        {
            dbg!(format!("{year}{month:0>2}{day:0>2}"))
        } else {
            s
        }
    } else {
        s
    }
}

fn validate(da: &str) -> bool {
    da.len() == 8 && da.chars().all(|c| c.is_numeric())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("1/2/1989", "19890102")]
    #[case("10/3/1989", "19891003")]
    #[case("11/24/1989", "19891124")]
    #[case("19891124", "19891124")]
    fn test_valid(#[case] input: &str, #[case] expected: &str) {
        let actual = DicomDate::try_new(input);
        assert_eq!(actual.map(|s| s.into_inner()).as_deref(), Ok(expected));
    }

    #[rstest]
    #[case("")]
    #[case("1234567")]
    #[case("text")]
    fn test_invalid(#[case] input: &str) {
        assert!(DicomDate::try_new(input).is_err())
    }
}
