use itertools::Itertools;
use wasm_bindgen::prelude::*;

/// Check credentials strength.
#[wasm_bindgen(js_name = "checkCredentialsStrength")]
pub fn check_credentials_strength(username: &str, password: &str) -> Result<(), JsError> {
    if strsim::levenshtein(username, password) < 5 {
        return Err(JsError::new("Username and password are too similar"));
    }

    if password.chars().any(|c| !c.is_ascii()) {
        return Err(JsError::new("Password contains non ASCII characters"));
    }

    let charset: String = password.chars().unique().collect();
    let mut total_range = 0;
    if charset.chars().any(|c| c.is_ascii_digit()) {
        total_range += 10;
    }
    if charset.chars().any(|c| c.is_ascii_lowercase()) {
        total_range += 26;
    }
    if charset.chars().any(|c| c.is_ascii_uppercase()) {
        total_range += 26;
    }
    if charset.chars().any(|c| !c.is_ascii_alphanumeric()) {
        total_range += 32;
    }

    let entropy = charset.len() as f64 * (total_range as f64).log2();
    if entropy < 75.0 {
        return Err(JsError::new("Too weak password!"));
    }

    Ok(())
}
