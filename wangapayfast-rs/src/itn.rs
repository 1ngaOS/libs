use std::collections::BTreeMap;

use url::form_urlencoded;

use crate::error::{Error, Result};

/// Configuration for verifying PayFast ITN signatures.
///
/// Typically you will construct this once at startup and reuse it for all ITN
/// requests.
#[derive(Debug, Clone)]
pub struct PayFastConfig {
    /// Optional passphrase configured in your PayFast merchant account.
    ///
    /// If you do not use a passphrase, leave this as `None`.
    pub passphrase: Option<String>,
}

impl PayFastConfig {
    /// Build a new config with an optional passphrase.
    pub fn new(passphrase: Option<impl Into<String>>) -> Self {
        Self {
            passphrase: passphrase.map(Into::into),
        }
    }
}

/// A parsed PayFast ITN request.
///
/// This holds:
/// - all fields from the ITN payload (except `signature`)
/// - the `signature` that was sent by PayFast
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItnRequest {
    params: BTreeMap<String, String>,
    signature: String,
}

impl ItnRequest {
    /// Parse an ITN request from a raw `application/x-www-form-urlencoded`
    /// HTTP body.
    ///
    /// The `signature` field is extracted into [`ItnRequest::signature`] and
    /// removed from [`ItnRequest::params`]. All other fields are kept as
    /// strings.
    pub fn from_body(body: &[u8]) -> Result<Self> {
        let raw: BTreeMap<String, String> = serde_urlencoded::from_bytes(body)?;

        let mut params = BTreeMap::new();
        let mut signature: Option<String> = None;

        for (k, v) in raw.into_iter() {
            if k == "signature" {
                signature = Some(v);
            } else {
                params.insert(k, v);
            }
        }

        let signature = signature.ok_or(Error::MissingSignature)?;

        Ok(Self { params, signature })
    }

    /// All ITN fields except `signature`, sorted by key.
    pub fn params(&self) -> &BTreeMap<String, String> {
        &self.params
    }

    /// The `signature` value that was sent by PayFast.
    pub fn signature(&self) -> &str {
        &self.signature
    }

    /// Generate the expected signature for this ITN request, using the
    /// configured passphrase (if any).
    pub fn expected_signature(&self, config: &PayFastConfig) -> String {
        generate_itn_signature(&self.params, config.passphrase.as_deref())
    }

    /// Check whether the ITN signature is valid for the given configuration.
    pub fn is_valid(&self, config: &PayFastConfig) -> bool {
        verify_itn_signature(self, config.passphrase.as_deref())
    }
}

/// Generate the PayFast ITN signature for the given parameters.
///
/// - `params` should include **all** ITN fields **except** `signature`.
/// - `passphrase` is your PayFast passphrase, if configured.
///
/// The implementation follows the algorithm described in the PayFast
/// documentation:
///
/// 1. Remove any fields whose value is blank after trimming.
/// 2. Sort the remaining fields by key (lexicographically).
/// 3. URL‑encode each value as in HTML forms (`application/x-www-form-urlencoded`).
/// 4. Join into `key=value&key2=value2...`.
/// 5. If a passphrase is provided, append `&passphrase=your-passphrase`.
/// 6. Compute the MD5 hex digest.
pub fn generate_itn_signature(
    params: &BTreeMap<String, String>,
    passphrase: Option<&str>,
) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());

    for (key, value) in params {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        serializer.append_pair(key, trimmed);
    }

    if let Some(passphrase) = passphrase {
        serializer.append_pair("passphrase", passphrase);
    }

    let canonical = serializer.finish();

    let digest = md5::compute(canonical.as_bytes());
    format!("{:x}", digest)
}

/// Verify that the ITN signature matches what we expect for the given
/// parameters and passphrase.
pub fn verify_itn_signature(itn: &ItnRequest, passphrase: Option<&str>) -> bool {
    let expected = generate_itn_signature(&itn.params, passphrase);
    // PayFast signatures are lower‑case hex MD5. Be forgiving and compare
    // case‑insensitively.
    expected.eq_ignore_ascii_case(itn.signature())
}

