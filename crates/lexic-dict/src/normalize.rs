use crate::error::DictError;
use unicode_normalization::UnicodeNormalization;

pub fn normalize(raw: &str) -> Result<String, DictError> {
    let collapsed: String = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return Err(crate::error::DictError::EmptyQuery);
    }
    if collapsed.chars().any(char::is_control) {
        return Err(DictError::InvalidCharacter);
    }
    let normalized: String = collapsed.nfkc().collect::<String>().to_lowercase();
    if normalized.chars().count() > 64 {
        return Err(crate::error::DictError::TooLong);
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_lowercases() {
        assert_eq!(normalize("  HeLLo  ").unwrap(), "hello");
    }

    #[test]
    fn collapses_whitespace() {
        assert_eq!(normalize("a   b\tc").unwrap(), "a b c");
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(
            normalize("   ").unwrap_err(),
            crate::error::DictError::EmptyQuery
        );
    }

    #[test]
    fn rejects_too_long() {
        let long = "a".repeat(65);
        assert_eq!(
            normalize(&long).unwrap_err(),
            crate::error::DictError::TooLong
        );
    }
}
