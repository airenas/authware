use anyhow::Ok;
use rcgen::{generate_simple_self_signed, Certificate, CertifiedKey, KeyPair};

pub fn generate_certificates(host: &str) -> anyhow::Result<(Certificate, KeyPair)> {
    tracing::info!("Generating certificates for host: {}", host);
    let subject_alt_names = vec![host.to_string()];
    let CertifiedKey { cert, key_pair } = generate_simple_self_signed(subject_alt_names)?;
    tracing::info!("Certificates generated");
    Ok((cert, key_pair))
}
