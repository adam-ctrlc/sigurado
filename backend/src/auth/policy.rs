//! Password rules, enforced server-side so the UI is a convenience rather than
//! the only thing standing between a weak password and the database.

use crate::error::AppError;

pub const MIN_LEN: usize = 8;
pub const MAX_LEN: usize = 16;

/// A human-readable summary, so the API and the UI can say the same thing.
pub const REQUIREMENTS: &str =
    "8 to 16 characters, with at least one uppercase letter, one lowercase letter, and one symbol";

pub fn validate_password(password: &str) -> Result<(), AppError> {
    // Count in characters, not bytes, so a non-ASCII password is judged fairly.
    let length = password.chars().count();
    if length < MIN_LEN {
        return Err(AppError::Validation(format!(
            "password must be at least {MIN_LEN} characters"
        )));
    }
    if length > MAX_LEN {
        return Err(AppError::Validation(format!(
            "password must be at most {MAX_LEN} characters"
        )));
    }
    if password.chars().any(char::is_whitespace) {
        return Err(AppError::Validation(
            "password cannot contain spaces".to_owned(),
        ));
    }
    if !password.chars().any(char::is_uppercase) {
        return Err(AppError::Validation(
            "password needs at least one uppercase letter".to_owned(),
        ));
    }
    if !password.chars().any(char::is_lowercase) {
        return Err(AppError::Validation(
            "password needs at least one lowercase letter".to_owned(),
        ));
    }
    // Anything that is not a letter or a digit counts as a symbol.
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(AppError::Validation(
            "password needs at least one symbol, for example ! ? @ or #".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_password;

    #[test]
    fn accepts_a_compliant_password() {
        assert!(validate_password("Lab$tock1").is_ok());
    }

    #[test]
    fn rejects_the_obvious_failures() {
        assert!(validate_password("Ab$1").is_err(), "too short");
        assert!(validate_password("Abcdefghij$1234567").is_err(), "too long");
        assert!(validate_password("lab$tock1").is_err(), "no uppercase");
        assert!(validate_password("LAB$TOCK1").is_err(), "no lowercase");
        assert!(validate_password("LabStock1").is_err(), "no symbol");
        assert!(validate_password("Lab $tock").is_err(), "has a space");
    }
}
