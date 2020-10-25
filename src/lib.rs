use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, thiserror::Error, PartialEq)]
enum Error {
    #[error("failed to parse int: {0}")]
    ParseInt(#[from] ParseIntError),
    #[error("overflow during sum")]
    Overflow,
    #[error("unexpected end of string")]
    UnexpectedEndOfString,
}

#[allow(unused)]
fn add(input: &str) -> Result<u64, Error> {
    if input.is_empty() {
        return Ok(0);
    }

    let (delimiter, input) = if let Some(input) = input.strip_prefix("//") {
        let (idx, delimiter) = input
            .char_indices()
            .find(|(_, c)| *c == '\n')
            .ok_or(Error::UnexpectedEndOfString)?;
        input.split_at(idx)
    } else {
        (",", input)
    };

    input
        .split(delimiter)
        .flat_map(|s| s.split('\n'))
        .filter(|s| !s.is_empty())
        .try_fold::<_, _, Result<_, Error>>(0u64, |acc, s| {
            let n = u64::from_str(s)?;
            acc.checked_add(n).ok_or(Error::Overflow)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use proptest::prelude::*;

    #[test]
    fn add_empty_string_is_0() {
        assert_eq!(Ok(0), add(""));
    }

    type Sum = u64;

    fn generate_input(numbers: &[u64], sep: &str) -> (Option<Sum>, String) {
        let separators_seq = vec![sep, "\n"]
            .into_iter()
            .cycle()
            .take(numbers.len().saturating_sub(1));

        let numbers_seq = numbers
            .iter()
            .map(|n| n.to_string())
            .interleave(separators_seq.map(|s| s.to_owned()))
            .join("");

        let input = if sep == "," {
            numbers_seq
        } else {
            format!("//{}\n{}", sep, numbers_seq)
        };

        let sum = numbers.iter().try_fold(0u64, |acc, n| acc.checked_add(*n));

        (sum, input)
    }

    proptest! {
        #[test]
        fn single_number_sum_is_that_number(n: u64) {
            prop_assert_eq!(Ok(n), add(&n.to_string()));
        }

        #[test]
        fn not_a_number_fails(input in "[[:alpha:]]+") {
            prop_assert!(add(&input).is_err());
        }

        #[test]
        fn sum_several_numbers(numbers: Vec<u64>) {
            let (opt_sum, input) = generate_input(&numbers, ",");
            let res = add(&input);

            if let Some(sum) = opt_sum {
                prop_assert_eq!(Ok(sum), res);
            } else {
                prop_assert_eq!(Err(Error::Overflow), res);
            }
        }

        #[test]
        fn custom_separator_at_the_beginning(
            numbers in prop::collection::vec(prop::num::u64::ANY, 0usize..1000),
            sep in "[[:alpha:]]|[[:punct:]]"
        ) {
            let (opt_sum, input) = generate_input(&numbers, &sep);
            let res = add(&input);

            if let Some(sum) = opt_sum {
                prop_assert_eq!(Ok(sum), res);
            } else {
                prop_assert_eq!(Err(Error::Overflow), res);
            }
        }

        #[test]
        fn custom_multiline_separator_at_the_beginning(
            numbers in prop::collection::vec(prop::num::u64::ANY, 0usize..1000),
            sep in "([[:alpha:]]|[[:punct:]])+"
        ) {
            let (opt_sum, input) = generate_input(&numbers, &sep);
            let res = add(&input);

            if let Some(sum) = opt_sum {
                prop_assert_eq!(Ok(sum), res);
            } else {
                prop_assert_eq!(Err(Error::Overflow), res);
            }
        }
    }
}
