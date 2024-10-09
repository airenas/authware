/// Test if the SecretString does not reveal the secret
///
/// ```compile_fail
/// use authware::utils::secret_str::SecretString;
/// let x: SecretString = "".into();
/// println!("{:?}", x);
/// ```
///
/// ```compile_fail
/// use authware::utils::secret_str::SecretString;
/// let x: SecretString = "".into();
/// println!("{}", x);
/// ```

#[derive(Clone, PartialEq)]
pub struct SecretString(String);

impl SecretString {
    /// ```
    /// use authware::utils::secret_str::SecretString;
    ///
    /// let x: SecretString = "abc123".into();
    /// assert_eq!(x.reveal_secret(), "abc123");
    /// ```
    pub fn reveal_secret(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SecretString {
    fn from(secret: &str) -> Self {
        SecretString(secret.to_string())
    }
}

impl From<String> for SecretString {
    fn from(secret: String) -> Self {
        SecretString(secret)
    }
}
