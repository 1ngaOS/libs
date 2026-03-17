use std::collections::BTreeMap;

use url::form_urlencoded;

use crate::error::{Error, Result};

/// Configuration for verifying PayFast signatures (ITN + checkout).
///
/// Typically you will construct this once at startup and reuse it for all
/// requests.
#[derive(Debug, Clone)]
pub struct PayFastConfig {
    /// Optional passphrase configured in your PayFast merchant account.
    ///
    /// If you do not use a passphrase, leave this as `None`.
    pub passphrase: Option<String>,
    /// The merchant id you expect for payments to your account.
    pub merchant_id: Option<String>,
    /// The merchant key you expect for payments to your account.
    pub merchant_key: Option<String>,
}

impl PayFastConfig {
    /// Build a new config with just a passphrase.
    pub fn new(passphrase: Option<impl Into<String>>) -> Self {
        Self {
            passphrase: passphrase.map(Into::into),
            merchant_id: None,
            merchant_key: None,
        }
    }

    /// Also set the merchant id and key you expect.
    pub fn with_merchant(
        mut self,
        merchant_id: impl Into<String>,
        merchant_key: impl Into<String>,
    ) -> Self {
        self.merchant_id = Some(merchant_id.into());
        self.merchant_key = Some(merchant_key.into());
        self
    }
}

/// A low‑level parsed PayFast ITN request (all fields as strings).
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
}

/// High‑level view over an ITN request with helpers for common fields.
#[derive(Debug, Clone)]
pub struct ItnNotification {
    /// Raw ITN data (all fields as strings).
    pub raw: ItnRequest,
}

impl ItnNotification {
    /// Construct an [`ItnNotification`] from the raw HTTP body.
    pub fn from_body(body: &[u8]) -> Result<Self> {
        Ok(Self {
            raw: ItnRequest::from_body(body)?,
        })
    }

    /// Convenience accessor for the low‑level params.
    pub fn params(&self) -> &BTreeMap<String, String> {
        self.raw.params()
    }

    /// The PayFast payment status, if present.
    pub fn payment_status(&self) -> ItnPaymentStatus {
        self.params()
            .get("payment_status")
            .map(|s| s.as_str())
            .unwrap_or_default()
            .into()
    }

    /// The PayFast payment_method, if present.
    pub fn payment_method(&self) -> PaymentMethod {
        self.params()
            .get("payment_method")
            .map(|s| s.as_str())
            .unwrap_or_default()
            .into()
    }

    /// `true` if the ITN amount_gross matches the expected string amount.
    ///
    /// PayFast sends amounts as strings with 2 decimal places.
    pub fn is_gross_amount(&self, expected: &str) -> bool {
        self.params()
            .get("amount_gross")
            .map(|v| v == expected)
            .unwrap_or(false)
    }

    /// Check whether the ITN belongs to the expected merchant configured on
    /// [`PayFastConfig`].
    pub fn is_expected_merchant(&self, cfg: &PayFastConfig) -> bool {
        match (&cfg.merchant_id, &cfg.merchant_key) {
            (Some(id), Some(key)) => {
                self.params().get("merchant_id") == Some(id)
                    && self.params().get("merchant_key") == Some(key)
            }
            _ => true,
        }
    }
}

/// Normalised PayFast payment_status values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ItnPaymentStatus {
    /// The payment has been completed and funds should be treated as paid.
    Complete,
    /// The payment has been cancelled by the user or provider.
    Cancelled,
    /// The payment is still pending.
    Pending,
    /// Any other / unknown status string.
    #[default]
    Other,
}

impl From<&str> for ItnPaymentStatus {
    fn from(s: &str) -> Self {
        match s.to_ascii_uppercase().as_str() {
            "COMPLETE" => ItnPaymentStatus::Complete,
            "CANCELLED" => ItnPaymentStatus::Cancelled,
            "PENDING" => ItnPaymentStatus::Pending,
            _ => ItnPaymentStatus::Other,
        }
    }
}

impl ItnPaymentStatus {
    /// Returns `true` if this status represents a completed payment.
    pub fn is_complete(self) -> bool {
        matches!(self, ItnPaymentStatus::Complete)
    }

    /// Returns `true` if this status represents a cancelled payment.
    pub fn is_cancelled(self) -> bool {
        matches!(self, ItnPaymentStatus::Cancelled)
    }

    /// Returns `true` if this status represents a pending payment.
    pub fn is_pending(self) -> bool {
        matches!(self, ItnPaymentStatus::Pending)
    }
}

/// Normalised PayFast `payment_method` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaymentMethod {
    /// Payflex (Buy Now, Pay Later).
    Payflex,
    /// Google Pay.
    GooglePay,
    /// Capitec Pay.
    CapitecPay,
    /// Samsung Pay.
    SamsungPay,
    /// Apple Pay.
    ApplePay,
    /// Mukuru.
    Mukuru,
    /// Store card.
    StoreCard,
    /// MoreTyme.
    MoreTyme,
    /// Zapper.
    Zapper,
    /// SnapScan.
    SnapScan,
    /// SCode.
    SCode,
    /// MobiCred.
    MobiCred,
    /// Masterpass Scan to Pay.
    Masterpass,
    /// Debit card.
    DebitCard,
    /// Credit card.
    CreditCard,
    /// EFT.
    Eft,
    /// Other / unknown method string.
    #[default]
    Other,
}

impl From<&str> for PaymentMethod {
    fn from(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "pf" | "payflex" => PaymentMethod::Payflex,
            "gp" | "googlepay" | "google_pay" => PaymentMethod::GooglePay,
            "cp" | "capitecpay" | "capitec_pay" => PaymentMethod::CapitecPay,
            "sp" | "samsungpay" | "samsung_pay" => PaymentMethod::SamsungPay,
            "ap" | "applepay" | "apple_pay" => PaymentMethod::ApplePay,
            "mu" | "mukuru" => PaymentMethod::Mukuru,
            "rc" | "storecard" | "store_card" => PaymentMethod::StoreCard,
            "mt" | "moretyme" | "more_tyme" => PaymentMethod::MoreTyme,
            "zp" | "zapper" => PaymentMethod::Zapper,
            "ss" | "snapscan" | "snap_scan" => PaymentMethod::SnapScan,
            "sc" | "scode" | "s_code" => PaymentMethod::SCode,
            "mc" | "mobicred" | "mobi_cred" => PaymentMethod::MobiCred,
            "mp" | "masterpass" | "masterpass_scan_to_pay" => PaymentMethod::Masterpass,
            "dc" | "debitcard" | "debit_card" => PaymentMethod::DebitCard,
            "cc" | "creditcard" | "credit_card" | "card" => PaymentMethod::CreditCard,
            "ef" | "eft" => PaymentMethod::Eft,
            _ => PaymentMethod::Other,
        }
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

/// The canonical field order for checkout signatures as per PayFast docs.
///
/// You can customise this list if needed, but the default should match the
/// order in the "Create your checkout form" section.
#[derive(Debug, Clone)]
pub struct CheckoutFieldOrder(pub Vec<String>);

impl Default for CheckoutFieldOrder {
    fn default() -> Self {
        // Based on public examples; users can override if PayFast updates docs.
        let fields = vec![
            // Merchant Details
            "merchant_id",
            "merchant_key",
            "return_url",
            "cancel_url",
            "notify_url",
            "notify_method",
            "fica_id",
            // Buyer Detail
            "name_first",
            "name_last",
            "email_address",
            "cell_number",
            // Transaction Details
            "m_payment_id",
            "amount",
            "item_name",
            "item_description",
            "custom_int1",
            "custom_int2",
            "custom_int3",
            "custom_int4",
            "custom_int5",
            "custom_str1",
            "custom_str2",
            "custom_str3",
            "custom_str4",
            "custom_str5",
            // Transaction Options
            "email_confirmation",
            "confirmation_address",
            "currency",
            // Set Payment Method
            "payment_method",
            // Recurring Billing Details
            "subscription_type",
            "billing_date",
            "recurring_amount",
            "frequency",
            "cycles",
            "subscription_notify_email",
            "subscription_notify_webhook",
            "subscription_notify_buyer",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();

        Self(fields)
    }
}

/// Input for generating a checkout signature.
pub type CheckoutParams = BTreeMap<String, String>;

/// Generate a PayFast checkout signature from arbitrary parameters.
///
/// This is similar to [`generate_itn_signature`] but follows an explicit field
/// order as recommended for checkout forms.
pub fn generate_checkout_signature(
    params: &CheckoutParams,
    passphrase: Option<&str>,
    order: &CheckoutFieldOrder,
) -> String {
    // Filter out blank values and remove surrounding spaces.
    let mut filtered: BTreeMap<String, String> = BTreeMap::new();
    for (k, v) in params {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            continue;
        }
        filtered.insert(k.clone(), trimmed.to_string());
    }

    // IMPORTANT: PayFast checkout signature uses a fixed field order, and
    // fields not in the canonical list are excluded (e.g. `setup` for split
    // payments). This matches the official SDK behaviour.
    let keys: Vec<&str> = order
        .0
        .iter()
        .map(|s| s.as_str())
        .filter(|k| filtered.contains_key(*k))
        .collect();

    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for key in keys {
        // `setup` (split payments) is not included in signature as per PayFast docs.
        if key == "signature" || key == "setup" {
            continue;
        }
        if let Some(value) = filtered.get(key) {
            serializer.append_pair(key, value);
        }
    }

    if let Some(passphrase) = passphrase {
        serializer.append_pair("passphrase", passphrase);
    }

    let canonical = serializer.finish();
    let digest = md5::compute(canonical.as_bytes());
    format!("{:x}", digest)
}

/// Generate a checkout signature, enforcing PayFast rules that require a
/// passphrase for some functionality (e.g. subscriptions / tokenization).
pub fn try_generate_checkout_signature(
    params: &CheckoutParams,
    passphrase: Option<&str>,
    order: &CheckoutFieldOrder,
) -> Result<String> {
    let is_subscription = params
        .get("subscription_type")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);

    if is_subscription && passphrase.map(|p| p.trim().is_empty()).unwrap_or(true) {
        return Err(Error::Validation(
            "subscriptions require a passphrase to be set".to_string(),
        ));
    }

    Ok(generate_checkout_signature(params, passphrase, order))
}

/// Result of an HTTP post‑back validation to PayFast.
#[cfg(feature = "http")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItnValidationOutcome {
    /// PayFast confirmed the ITN as valid.
    Valid,
    /// PayFast explicitly rejected the ITN.
    Invalid,
    /// Network / protocol error while attempting validation.
    NetworkError,
}

/// Post the ITN data back to PayFast for validation (optional, requires
/// feature `http`).
///
/// This implements the "post back" step described in the PayFast docs:
/// you send the same form body back to PayFast's validation endpoint and
/// check the response.
#[cfg(feature = "http")]
pub async fn post_back_validate_itn(
    client: &reqwest::Client,
    body: &[u8],
    sandbox: bool,
) -> ItnValidationOutcome {
    let url = if sandbox {
        "https://sandbox.payfast.co.za/eng/query/validate"
    } else {
        "https://www.payfast.co.za/eng/query/validate"
    };

    let res = client
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body.to_vec())
        .send()
        .await;

    let Ok(resp) = res else {
        return ItnValidationOutcome::NetworkError;
    };

    let Ok(text) = resp.text().await else {
        return ItnValidationOutcome::NetworkError;
    };

    if text.trim().eq_ignore_ascii_case("VALID") {
        ItnValidationOutcome::Valid
    } else if text.trim().eq_ignore_ascii_case("INVALID") {
        ItnValidationOutcome::Invalid
    } else {
        ItnValidationOutcome::NetworkError
    }
}
