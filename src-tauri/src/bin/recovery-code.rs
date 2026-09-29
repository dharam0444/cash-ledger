use sha2::{Digest, Sha256};

const VENDOR_LICENSE_SECRET: &str = "CHANGE_THIS_VENDOR_SECRET_BEFORE_RELEASE_2026";

fn main() {
    let machine_code = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: cargo run --bin recovery-code -- MACHINE-CODE");
        std::process::exit(2);
    });
    let text = format!(
        "{}|RECOVERY|{}",
        VENDOR_LICENSE_SECRET,
        machine_code.trim().to_uppercase()
    );
    let hash = hex::encode_upper(Sha256::digest(text.as_bytes()));
    println!("{}-{}-{}-{}", &hash[0..8], &hash[8..16], &hash[16..24], &hash[24..32]);
}
