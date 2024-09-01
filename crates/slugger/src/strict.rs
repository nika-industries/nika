//! Strict slugify implementation

/// Convert any unicode string to a "strict slug" (useful for any user-defined
/// name that might be used in a URL component)
///
/// The returned "slug" will consist of a-z, 0-9, and '-'. Furthermore, a slug
/// will never contain more than one '-' in a row and will never start or end
/// with '-'.
pub fn strict_slugify(s: &str) -> String {
  let mut slug = String::with_capacity(s.len());
  // Starts with true to avoid leading -
  let mut prev_is_dash = true;

  for c in s.chars() {
    if c.is_ascii() {
      push_char(&mut slug, c as u8, &mut prev_is_dash);
    } else {
      for &cx in deunicode::deunicode_char(c).unwrap_or("-").as_bytes() {
        push_char(&mut slug, cx, &mut prev_is_dash);
      }
    }
  }

  if slug.ends_with('-') {
    slug.pop();
  }
  // We likely reserved more space than needed.
  slug.shrink_to_fit();
  slug
}

#[inline]
fn push_char(slug: &mut String, x: u8, prev_is_dash: &mut bool) {
  match x {
    b'a'..=b'z' | b'0'..=b'9' => {
      *prev_is_dash = false;
      slug.push(x.into());
    }
    b'A'..=b'Z' => {
      *prev_is_dash = false;
      // Manual lowercasing as Rust to_lowercase() is unicode
      // aware and therefore much slower
      slug.push((x - b'A' + b'a').into());
    }
    _ => {
      if !*prev_is_dash {
        slug.push('-');
        *prev_is_dash = true;
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_strict_slugify_basic() {
    // Basic ASCII input
    assert_eq!(strict_slugify("hello-world"), "hello-world");
    assert_eq!(strict_slugify("Hello World"), "hello-world");
    assert_eq!(strict_slugify("rust+rocks"), "rust-rocks");
  }

  #[test]
  fn test_strict_slugify_unicode() {
    // Unicode characters that should be replaced with dashes
    assert_eq!(strict_slugify("你好世界"), "ni-hao-shi-jie");
    assert_eq!(strict_slugify("こんにちは"), "konnitiha");
    assert_eq!(strict_slugify("¡Hola!"), "hola");
  }

  #[test]
  fn test_strict_slugify_mixed() {
    // Mixed ASCII and Unicode
    assert_eq!(strict_slugify("rust编程语言"), "rustbian-cheng-yu-yan");
    assert_eq!(strict_slugify("Lörem Ipsum"), "lorem-ipsum");
    assert_eq!(strict_slugify("foo@bar.com"), "foo-bar-com");
  }

  #[test]
  fn test_strict_slugify_special_characters() {
    // Input with special characters
    assert_eq!(strict_slugify("foo_bar.baz+qux"), "foo-bar-baz-qux");
    assert_eq!(strict_slugify("foo/bar\\baz"), "foo-bar-baz");
    assert_eq!(strict_slugify("hello*world"), "hello-world");
  }

  #[test]
  fn test_strict_slugify_edge_cases() {
    // Edge cases
    assert_eq!(strict_slugify(""), "");
    assert_eq!(strict_slugify("  "), "");
    assert_eq!(strict_slugify("..."), "");
    assert_eq!(strict_slugify("a"), "a");
    assert_eq!(strict_slugify("-_-"), "");
  }
}
