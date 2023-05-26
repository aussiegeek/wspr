use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum EncodeError {
    #[error("Invalid alphanumeric char: {0}")]
    InvalidChar(u8),
    #[error("Invalid callsign")]
    InvalidCallsign,
}

// encode a byte that could be 0-9 or A-Z
pub(crate) fn encode_num_str(char: u8) -> Result<u64, EncodeError> {
    match char {
        b'0'..=b'9' => Ok((char - 48).into()),
        b'A'..=b'Z' => Ok((char - 55).into()),
        b' ' => Ok(36),
        _ => Err(EncodeError::InvalidChar(char)),
    }
}

pub(crate) fn encode_locator_char(char: u8) -> Result<u32, EncodeError> {
    match char {
        b'A'..=b'R' => Ok((char - 65).into()),
        _ => Err(EncodeError::InvalidChar(char)),
    }
}

#[cfg(test)]
mod tests {
    use crate::value_encode::{encode_locator_char, encode_num_str};

    #[test]
    fn test_encode_num_str() {
        assert_eq!(encode_num_str(b'0'), Ok(0));
        assert_eq!(encode_num_str(b'1'), Ok(1));
        assert_eq!(encode_num_str(b'9'), Ok(9));
        assert_eq!(encode_num_str(b'A'), Ok(10));
        assert_eq!(encode_num_str(b'X'), Ok(33));
        assert_eq!(encode_num_str(b'Z'), Ok(35));
        assert_eq!(encode_num_str(b' '), Ok(36));
    }

    #[test]
    fn test_encode_locator_char() {
        assert_eq!(encode_locator_char(b'A'), Ok(0));
        assert_eq!(encode_locator_char(b'F'), Ok(5));
        assert_eq!(encode_locator_char(b'Q'), Ok(16));
        assert_eq!(encode_locator_char(b'R'), Ok(17));
    }
}
