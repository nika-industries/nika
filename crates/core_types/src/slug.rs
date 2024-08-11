pub use slug::slugify;

/// Determines whether a string is a valid slug. Based on [`slugify()`].
pub fn is_slug<S: AsRef<str>>(s: S) -> bool {
  let s = s.as_ref();
  let bytes = s.as_bytes();
  let len = bytes.len();

  // Early return if the string is empty or has leading/trailing hyphens
  if len == 0 || bytes[0] == b'-' || bytes[len - 1] == b'-' {
    return false;
  }

  // Check the characters and avoid consecutive hyphens
  for i in 0..len {
    match bytes[i] {
      b'a'..=b'z' | b'0'..=b'9' => continue,
      b'-' if i > 0 && bytes[i - 1] != b'-' => continue,
      _ => return false,
    }
  }

  true
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_valid_slugs() {
    assert!(is_slug("valid-slug"));
    assert!(is_slug("a"));
    assert!(is_slug("a1-b2-c3"));
    assert!(is_slug("simple-slug-123"));
    assert!(is_slug("lowercase-letters"));
    assert!(is_slug("12345"));
  }

  #[test]
  fn test_empty_string() {
    assert!(!is_slug(""));
  }

  #[test]
  fn test_leading_trailing_hyphens() {
    assert!(!is_slug("-invalid"));
    assert!(!is_slug("invalid-"));
    assert!(!is_slug("-"));
    assert!(!is_slug("-leading-and-trailing-"));
  }

  #[test]
  fn test_consecutive_hyphens() {
    assert!(!is_slug("consecutive--hyphens"));
    assert!(!is_slug("multiple---hyphens"));
    assert!(!is_slug("a--b"));
  }

  #[test]
  fn test_invalid_characters() {
    assert!(!is_slug("Invalid-Caps"));
    assert!(!is_slug("invalid_slug"));
    assert!(!is_slug("invalid@slug!"));
    assert!(!is_slug("invalid slug"));
    assert!(!is_slug("invalid.slug"));
    assert!(!is_slug("invalid/slug"));
    assert!(!is_slug("invalid\\slug"));
  }

  #[test]
  fn test_non_ascii_characters() {
    assert!(!is_slug("slugé"));
    assert!(!is_slug("slug中文"));
    assert!(!is_slug("slug-€"));
  }

  #[test]
  fn test_numeric_slugs() {
    assert!(is_slug("12345"));
    assert!(is_slug("1-2-3"));
    assert!(!is_slug("1-2--3"));
  }

  #[test]
  fn test_mixed_alphanumeric_slugs() {
    assert!(is_slug("abc123"));
    assert!(is_slug("abc-123"));
    assert!(is_slug("a1-b2-c3"));
  }
}
