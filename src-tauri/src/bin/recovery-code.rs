use offline_cash_ledger_lib::recovery_code::generate_recovery_code;
use rand_core::{OsRng, RngCore};

fn main() {
    let machine_code = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: cargo run --bin recovery-code -- MACHINE-CODE");
        std::process::exit(2);
    });
    let mut nonce = [0_u8; 8];
    OsRng.fill_bytes(&mut nonce);
    println!("{}", generate_recovery_code(&machine_code, nonce));
}
