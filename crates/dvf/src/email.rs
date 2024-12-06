use std::{str::FromStr, sync::LazyLock};

/// Our "reasonable" email regex is significantly more restrictive than the
/// RFCs. sourced from https://colinhacks.com/essays/reasonable-email-regex,
/// with lowercase ranges added.
const EMAIL_REASONABLE_REGEX_STRING: &str = r#"^([A-Za-z0-9_+-]+\.?)*[A-Za-z0-9_+-]@([A-Za-z0-9][A-Za-z0-9-]*\.)+[A-Za-z]{2,}$"#;
static EMAIL_REASONABLE_REGEX: LazyLock<regex::Regex> =
  LazyLock::new(|| regex::Regex::new(EMAIL_REASONABLE_REGEX_STRING).unwrap());

/// Validate that an email address is technically compliant.
pub fn validate_compliant_email_address(email: &str) -> bool {
  email_address::EmailAddress::from_str(email).is_ok()
}

/// Validate that an email address is reasonable.
pub fn validate_reasonable_email_address(email: &str) -> bool {
  EMAIL_REASONABLE_REGEX.is_match(email)
}

#[nutype::nutype(
  derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    AsRef,
    Display
  ),
  validate(predicate = validate_compliant_email_address, len_char_max = 128)
)]
pub struct EmailAddress(String);

impl EmailAddress {
  /// Check if the email address is reasonable.
  pub fn is_reasonable(&self) -> bool {
    validate_reasonable_email_address(self.as_ref())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn basic_email_test() {
    let reasonable_email = "bob@example.com";
    let compliant_email = "main@[192.168.0.1]";
    assert!(validate_compliant_email_address(compliant_email));
    assert!(validate_compliant_email_address(reasonable_email));
    assert!(validate_reasonable_email_address(reasonable_email));
    assert!(!validate_reasonable_email_address(compliant_email));
  }
}
