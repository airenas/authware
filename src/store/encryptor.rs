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
