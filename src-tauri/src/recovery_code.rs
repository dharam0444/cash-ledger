use hmac::{Hmac, Mac};
use sha2::Sha256;

const VENDOR_LICENSE_SECRET: &str = "CHANGE_THIS_VENDOR_SECRET_BEFORE_RELEASE_2026";
const NONCE_BYTES: usize = 8;
const SIGNATURE_BYTES: usize = 8;

/// Creates a recovery code bound to `machine_code`.
///
/// The nonce must be freshly generated with a cryptographically secure random
/// number generator for every recovery request. It is embedded in the code;
/// the remaining bytes are an HMAC that prevents a client from forging one.
pub fn generate_recovery_code(machine_code: &str, nonce: [u8; NONCE_BYTES]) -> String {
    let signature = recovery_signature(machine_code, &nonce);
    let mut code = Vec::with_capacity(NONCE_BYTES + SIGNATURE_BYTES);
    code.extend_from_slice(&nonce);
    code.extend_from_slice(&signature);
    format_code(&code)
}

pub fn validate_recovery_code(machine_code: &str, code: &str) -> bool {
    let normalized = normalize_code(code);
    let expected_length = (NONCE_BYTES + SIGNATURE_BYTES) * 2;
    if normalized.len() != expected_length
        || !normalized
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return false;
    }

    let Ok(bytes) = hex::decode(normalized) else {
        return false;
    };
    let (nonce, signature) = bytes.split_at(NONCE_BYTES);

    let mut mac = Hmac::<Sha256>::new_from_slice(VENDOR_LICENSE_SECRET.as_bytes())
        .expect("HMAC accepts keys of any length");
    mac.update(b"RECOVERY|");
    mac.update(machine_code.trim().to_uppercase().as_bytes());
    mac.update(b"|");
    mac.update(nonce);
    mac.verify_truncated_left(signature).is_ok()
}

fn recovery_signature(machine_code: &str, nonce: &[u8; NONCE_BYTES]) -> [u8; SIGNATURE_BYTES] {
    let mut mac = Hmac::<Sha256>::new_from_slice(VENDOR_LICENSE_SECRET.as_bytes())
        .expect("HMAC accepts keys of any length");
    mac.update(b"RECOVERY|");
    mac.update(machine_code.trim().to_uppercase().as_bytes());
    mac.update(b"|");
    mac.update(nonce);

    let digest = mac.finalize().into_bytes();
    let mut signature = [0_u8; SIGNATURE_BYTES];
    signature.copy_from_slice(&digest[..SIGNATURE_BYTES]);
    signature
}

fn format_code(bytes: &[u8]) -> String {
    let hex = hex::encode_upper(bytes);
    (0..hex.len())
        .step_by(8)
        .map(|start| &hex[start..start + 8])
        .collect::<Vec<_>>()
        .join("-")
}

fn normalize_code(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::{generate_recovery_code, validate_recovery_code};

    #[test]
    fn valid_code_is_bound_to_its_machine() {
        let code = generate_recovery_code("2169F3-34E2DE-B3281F", [1, 2, 3, 4, 5, 6, 7, 8]);

        assert!(validate_recovery_code("2169F3-34E2DE-B3281F", &code));
        assert!(!validate_recovery_code("OTHER-MACHINE", &code));
    }

    #[test]
    fn distinct_nonces_create_distinct_codes() {
        let first = generate_recovery_code("2169F3-34E2DE-B3281F", [0; 8]);
        let second = generate_recovery_code("2169F3-34E2DE-B3281F", [1; 8]);

        assert_ne!(first, second);
    }
}
