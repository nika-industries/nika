//! Strict slugify implementation

/// Convert any unicode string to an ascii "slug" (useful for file names/url
/// components)
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
  fn test_strict_slugify() {
    assert_eq!(strict_slugify("My Test String!!!1!1"), "my-test-string-1-1");
    assert_eq!(strict_slugify("test\nit   now!"), "test-it-now");
    assert_eq!(strict_slugify("  --test_-_cool"), "test-cool");
    assert_eq!(strict_slugify("Æúű--cool?"), "aeuu-cool");
    assert_eq!(strict_slugify("You & Me"), "you-me");
    assert_eq!(strict_slugify("user@example.com"), "user-example-com");
  }
}
