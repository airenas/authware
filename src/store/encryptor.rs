use magic_crypt::MagicCryptTrait;

use crate::Encryptor;

pub struct MagicEncryptor {
    encryptor: magic_crypt::MagicCrypt256,
}

impl MagicEncryptor {
    pub fn new(key: &str) -> anyhow::Result<Self> {
        if key.len() < 16 {
            return Err(anyhow::anyhow!("encryption key length must >= 16"));
        }
        Ok(MagicEncryptor {
            encryptor: magic_crypt::new_magic_crypt!(key, 256),
        })
    }
}

impl Encryptor for MagicEncryptor {
    fn encrypt(&self, data: &str) -> String {
        self.encryptor.encrypt_str_to_base64(data)
    }

    fn decrypt(&self, data: &str) -> anyhow::Result<String> {
        self.encryptor
            .decrypt_base64_to_string(data)
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("", "", "DgXh9i/pIea5iXvXZg15dw=="; "empty")]
    #[test_case("olia", "", "LVX/HEA5MsQR0J9NZSNLLA=="; "value")]
    #[test_case("olia", "aaaaaa", "gkYxNAn3tL5JcgE4O8x7Zg=="; "value other key")]
    fn test_encrypt(input: &str, key_suffix: &str, expected: &str) {
        let key = "1234567890123456".to_string() + key_suffix;
        let encryptor = MagicEncryptor::new(&key).unwrap();
        let actual = encryptor.encrypt(input);
        assert_eq!(expected, actual);
    }

    #[test_case("DgXh9i/pIea5iXvXZg15dw==", "", ""; "empty")]
    #[test_case("LVX/HEA5MsQR0J9NZSNLLA==", "", "olia"; "value")]
    #[test_case("gkYxNAn3tL5JcgE4O8x7Zg==", "aaaaaa", "olia"; "value other key")]
    fn test_decrypt(input: &str, key_suffix: &str, expected: &str) {
        let key = "1234567890123456".to_string() + key_suffix;
        let encryptor = MagicEncryptor::new(&key).unwrap();
        let actual = encryptor.decrypt(input).unwrap();
        assert_eq!(expected, actual);
    }
}
