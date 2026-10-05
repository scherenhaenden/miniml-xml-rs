#![no_std]
#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertError {
    Empty,
    InvalidDigit,
    Overflow,
    Underflow,
    SignOnly,
}

/// Parses an XML schema integer (i64).
/// Allows optional leading +/- and digits after XML whitespace trim/collapse policy.
/// Rejects internal spaces, empty/sign-only/invalid digits, overflow/underflow.
pub fn parse_integer(input: &str) -> Result<i64, ConvertError> {
    let s = input.trim_matches(|c| c == ' ' || c == '\n' || c == '\r' || c == '\t');

    if s.is_empty() {
        return Err(ConvertError::Empty);
    }

    let mut chars = s.chars();
    let mut is_negative = false;
    let mut started_digits = false;
    let mut value: i64 = 0;

    let first = chars.next().unwrap();
    if first == '-' {
        is_negative = true;
    } else if first == '+' {
        // Just skip
    } else if first.is_ascii_digit() {
        started_digits = true;
        value = (first as u8 - b'0') as i64;
    } else {
        return Err(ConvertError::InvalidDigit);
    }

    for c in chars {
        if !c.is_ascii_digit() {
            return Err(ConvertError::InvalidDigit);
        }
        started_digits = true;
        let digit = (c as u8 - b'0') as i64;

        if is_negative {
            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_sub(digit))
                .ok_or(ConvertError::Underflow)?;
        } else {
            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_add(digit))
                .ok_or(ConvertError::Overflow)?;
        }
    }

    if !started_digits {
        return Err(ConvertError::SignOnly);
    }

    Ok(value)
}

/// Parses an XML schema string.
/// Currently minimal string validations. XML schema string validation mostly relates to facets which we don't support.
pub fn parse_string(input: &str) -> Result<&str, ConvertError> {
    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer_valid() {
        assert_eq!(parse_integer("123").unwrap(), 123);
        assert_eq!(parse_integer("+123").unwrap(), 123);
        assert_eq!(parse_integer("-123").unwrap(), -123);
        assert_eq!(parse_integer("0").unwrap(), 0);
        assert_eq!(parse_integer("-0").unwrap(), 0);
        assert_eq!(parse_integer("+0").unwrap(), 0);
        assert_eq!(parse_integer("00123").unwrap(), 123);
    }

    #[test]
    fn test_parse_integer_whitespace() {
        assert_eq!(parse_integer("  123  ").unwrap(), 123);
        assert_eq!(parse_integer("\t\n\r -123 \t\n\r ").unwrap(), -123);

        // Internal whitespace should fail
        assert_eq!(
            parse_integer("1 23").unwrap_err(),
            ConvertError::InvalidDigit
        );
        assert_eq!(
            parse_integer("- 123").unwrap_err(),
            ConvertError::InvalidDigit
        );
    }

    #[test]
    fn test_parse_integer_limits() {
        // i64::MAX = 9223372036854775807
        assert_eq!(parse_integer("9223372036854775807").unwrap(), i64::MAX);
        assert_eq!(
            parse_integer("9223372036854775808").unwrap_err(),
            ConvertError::Overflow
        );

        // i64::MIN = -9223372036854775808
        assert_eq!(parse_integer("-9223372036854775808").unwrap(), i64::MIN);
        assert_eq!(
            parse_integer("-9223372036854775809").unwrap_err(),
            ConvertError::Underflow
        );
    }

    #[test]
    fn test_parse_integer_invalid() {
        assert_eq!(parse_integer("").unwrap_err(), ConvertError::Empty);
        assert_eq!(parse_integer("   ").unwrap_err(), ConvertError::Empty);
        assert_eq!(parse_integer("+").unwrap_err(), ConvertError::SignOnly);
        assert_eq!(parse_integer("-").unwrap_err(), ConvertError::SignOnly);
        assert_eq!(
            parse_integer("123a").unwrap_err(),
            ConvertError::InvalidDigit
        );
        assert_eq!(
            parse_integer("a123").unwrap_err(),
            ConvertError::InvalidDigit
        );
        assert_eq!(
            parse_integer("+-123").unwrap_err(),
            ConvertError::InvalidDigit
        );
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_string("test").unwrap(), "test");
        assert_eq!(parse_string("  test  ").unwrap(), "  test  ");
        assert_eq!(parse_string("").unwrap(), "");
    }
}
