use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{generate_checkout_signature, CheckoutFieldOrder, CheckoutParams, PayFastConfig};

/// Split payment JSON payload (`setup`) for PayFast split payments.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SplitPaymentSetup {
    /// The split rule payload.
    pub split_payment: SplitPaymentRule,
}

/// Rule for how a payment is split with a third party merchant.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SplitPaymentRule {
    /// Third-party merchant id.
    pub merchant_id: u64,
    /// Fixed split amount in cents (optional if `percentage` is set).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<u64>,
    /// Percentage split (optional if `amount` is set).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<u64>,
    /// Minimum split amount in cents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<u64>,
    /// Maximum split amount in cents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<u64>,
}

/// High-level request for a once-off PayFast payment.
#[derive(Debug, Clone, Deserialize)]
pub struct OnceOffPaymentRequest {
    /// Your own internal payment / order id (mapped to `m_payment_id`).
    pub payment_id: String,
    /// Amount as a string with 2 decimal places, e.g. `"100.00"`.
    pub amount: String,
    /// Item name / order label.
    pub item_name: String,
    /// Optional description.
    pub item_description: Option<String>,

    /// Optional currency code (e.g. `"ZAR"`, `"USD"`) for integrations that
    /// support it.
    ///
    /// Mapped to the `currency` field, as used by the official PayFast SDKs.
    pub currency: Option<String>,

    /// Legacy alias for [`OnceOffPaymentRequest::currency`].
    ///
    /// If set, it is mapped to the same `currency` field.
    pub currency_code: Option<String>,

    /// Buyer first name.
    pub name_first: Option<String>,
    /// Buyer last name.
    pub name_last: Option<String>,
    /// Buyer email.
    pub email_address: Option<String>,
    /// Buyer cell number.
    pub cell_number: Option<String>,

    /// Return URL after payment.
    pub return_url: Option<String>,
    /// Cancel URL if the buyer cancels.
    pub cancel_url: Option<String>,
    /// Notify URL for ITN callbacks; if omitted you can configure this in the
    /// PayFast dashboard.
    pub notify_url: Option<String>,
    /// Optional notify method (`notify_method`).
    pub notify_method: Option<String>,
    /// Optional buyer FICA id number (`fica_id`) for SA ID numbers.
    pub fica_id: Option<String>,

    /// Optional arbitrary custom fields (mapped to `custom_str*/custom_int*` or
    /// any other supported PayFast field names).
    #[serde(default)]
    pub custom: BTreeMap<String, String>,

    /// Transaction option: email confirmation flag (`email_confirmation`).
    pub email_confirmation: Option<bool>,
    /// Transaction option: override confirmation email address (`confirmation_address`).
    pub confirmation_address: Option<String>,
}

/// Subscription / recurring payment options.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SubscriptionOptions {
    /// 1 for subscription, 2 for ad-hoc, etc. (`subscription_type`).
    pub subscription_type: Option<String>,
    /// First billing date (`billing_date`).
    pub billing_date: Option<String>,
    /// Recurring amount (`recurring_amount`).
    pub recurring_amount: Option<String>,
    /// Frequency in days / months depending on PayFast rules (`frequency`).
    pub frequency: Option<String>,
    /// Number of cycles (`cycles`).
    pub cycles: Option<String>,

    /// Send the merchant an email notification before a trial ends or amount increases
    /// (`subscription_notify_email`).
    pub subscription_notify_email: Option<bool>,
    /// Send the merchant a webhook notification before a trial ends or amount increases
    /// (`subscription_notify_webhook`).
    pub subscription_notify_webhook: Option<bool>,
    /// Send the buyer an email notification before a trial ends or amount increases
    /// (`subscription_notify_buyer`).
    pub subscription_notify_buyer: Option<bool>,
}

/// Split payment settings (high-level; encoded into custom fields).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SplitPayment {
    /// High-level representation of split meta; you can encode details in
    /// `custom` map below as needed for your integration.
    pub primary_receiver: Option<String>,
    /// Optional secondary receiver identifier for split payments.
    pub secondary_receiver: Option<String>,
    /// Optional amount to allocate to the secondary receiver.
    pub secondary_amount: Option<String>,
    /// JSON encoded split-payment payload for PayFast (`setup`).
    ///
    /// Per PayFast docs this is **not included** in the signature. The crate
    /// automatically excludes `setup` when generating checkout signatures.
    pub setup: Option<String>,
    /// Typed split-payment payload. If provided, it will be serialized into
    /// the `setup` field (unless `setup` is already set).
    pub setup_payload: Option<SplitPaymentSetup>,
    /// Extra key/value metadata for custom split encodings.
    #[serde(default)]
    pub custom: BTreeMap<String, String>,
}

/// Combined subscription + split once-off request for convenience.
#[derive(Debug, Clone, Deserialize)]
pub struct AdvancedPaymentRequest {
    #[serde(flatten)]
    /// Base once-off payment details (amount, item, buyer, URLs, etc.).
    pub base: OnceOffPaymentRequest,
    #[serde(default)]
    /// Optional subscription / recurring billing options.
    pub subscription: SubscriptionOptions,
    #[serde(default)]
    /// Optional split payment configuration.
    pub split: SplitPayment,
}

/// Response containing the URL and parameters for redirecting to PayFast.
#[derive(Debug, Clone, Serialize)]
pub struct CheckoutResponse {
    /// The PayFast process URL (sandbox or live).
    pub url: String,
    /// The form parameters (including `signature`) that must be posted.
    pub params: BTreeMap<String, String>,
}

/// Full custom integration checkout request (typed).
///
/// This structure mirrors the PayFast “Create your checkout form” field list,
/// plus recurring billing and split payments fields.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CheckoutRequest {
    // Merchant details
    /// The URL where the user is returned after successful payment (`return_url`).
    pub return_url: Option<String>,
    /// The URL where the user is returned after cancelling payment (`cancel_url`).
    pub cancel_url: Option<String>,
    /// The URL PayFast posts ITN notifications to (`notify_url`).
    pub notify_url: Option<String>,
    /// Notify method override (`notify_method`).
    pub notify_method: Option<String>,
    /// Buyer FICA id number (`fica_id`).
    pub fica_id: Option<String>,

    // Customer details
    /// Customer first name (`name_first`).
    pub name_first: Option<String>,
    /// Customer last name (`name_last`).
    pub name_last: Option<String>,
    /// Customer email address (`email_address`).
    pub email_address: Option<String>,
    /// Customer cell number (`cell_number`).
    pub cell_number: Option<String>,

    // Transaction details
    /// Merchant payment id (`m_payment_id`).
    pub m_payment_id: Option<String>,
    /// Payment amount (`amount`).
    pub amount: Option<String>,
    /// Item / order name (`item_name`).
    pub item_name: Option<String>,
    /// Item description (`item_description`).
    pub item_description: Option<String>,
    /// Custom integer pass-through (`custom_int1`).
    pub custom_int1: Option<String>,
    /// Custom integer pass-through (`custom_int2`).
    pub custom_int2: Option<String>,
    /// Custom integer pass-through (`custom_int3`).
    pub custom_int3: Option<String>,
    /// Custom integer pass-through (`custom_int4`).
    pub custom_int4: Option<String>,
    /// Custom integer pass-through (`custom_int5`).
    pub custom_int5: Option<String>,
    /// Custom string pass-through (`custom_str1`).
    pub custom_str1: Option<String>,
    /// Custom string pass-through (`custom_str2`).
    pub custom_str2: Option<String>,
    /// Custom string pass-through (`custom_str3`).
    pub custom_str3: Option<String>,
    /// Custom string pass-through (`custom_str4`).
    pub custom_str4: Option<String>,
    /// Custom string pass-through (`custom_str5`).
    pub custom_str5: Option<String>,

    // Transaction options
    /// Whether to send a merchant email confirmation (`email_confirmation`).
    pub email_confirmation: Option<bool>,
    /// Override confirmation email address (`confirmation_address`).
    pub confirmation_address: Option<String>,

    // Currency + method
    /// Currency code (`currency`).
    pub currency: Option<String>,
    /// Payment method code (`payment_method`), e.g. `cc`, `ef`, etc.
    pub payment_method: Option<String>,

    // Recurring billing
    /// Subscription type (`subscription_type`).
    pub subscription_type: Option<String>,
    /// Billing date (`billing_date`, `YYYY-MM-DD`).
    pub billing_date: Option<String>,
    /// Future recurring amount (`recurring_amount`).
    pub recurring_amount: Option<String>,
    /// Subscription frequency (`frequency`).
    pub frequency: Option<String>,
    /// Subscription cycles (`cycles`).
    pub cycles: Option<String>,
    /// Notify merchant by email (`subscription_notify_email`).
    pub subscription_notify_email: Option<bool>,
    /// Notify merchant by webhook (`subscription_notify_webhook`).
    pub subscription_notify_webhook: Option<bool>,
    /// Notify buyer by email (`subscription_notify_buyer`).
    pub subscription_notify_buyer: Option<bool>,

    // Split payments
    /// Split payments JSON payload (`setup`). Not included in signature.
    pub setup: Option<String>,
}

impl CheckoutRequest {
    fn into_params(self) -> CheckoutParams {
        let mut p = CheckoutParams::new();
        macro_rules! opt {
            ($k:literal, $v:expr) => {
                if let Some(v) = $v {
                    p.insert($k.to_string(), v);
                }
            };
        }
        macro_rules! opt_bool01 {
            ($k:literal, $v:expr) => {
                if let Some(v) = $v {
                    p.insert($k.to_string(), if v { "1" } else { "0" }.to_string());
                }
            };
        }

        opt!("return_url", self.return_url);
        opt!("cancel_url", self.cancel_url);
        opt!("notify_url", self.notify_url);
        opt!("notify_method", self.notify_method);
        opt!("fica_id", self.fica_id);

        opt!("name_first", self.name_first);
        opt!("name_last", self.name_last);
        opt!("email_address", self.email_address);
        opt!("cell_number", self.cell_number);

        opt!("m_payment_id", self.m_payment_id);
        opt!("amount", self.amount);
        opt!("item_name", self.item_name);
        opt!("item_description", self.item_description);
        opt!("custom_int1", self.custom_int1);
        opt!("custom_int2", self.custom_int2);
        opt!("custom_int3", self.custom_int3);
        opt!("custom_int4", self.custom_int4);
        opt!("custom_int5", self.custom_int5);
        opt!("custom_str1", self.custom_str1);
        opt!("custom_str2", self.custom_str2);
        opt!("custom_str3", self.custom_str3);
        opt!("custom_str4", self.custom_str4);
        opt!("custom_str5", self.custom_str5);

        opt_bool01!("email_confirmation", self.email_confirmation);
        opt!("confirmation_address", self.confirmation_address);

        opt!("currency", self.currency);
        opt!("payment_method", self.payment_method);

        opt!("subscription_type", self.subscription_type);
        opt!("billing_date", self.billing_date);
        opt!("recurring_amount", self.recurring_amount);
        opt!("frequency", self.frequency);
        opt!("cycles", self.cycles);
        opt_bool01!("subscription_notify_email", self.subscription_notify_email);
        opt_bool01!(
            "subscription_notify_webhook",
            self.subscription_notify_webhook
        );
        opt_bool01!("subscription_notify_buyer", self.subscription_notify_buyer);

        opt!("setup", self.setup);

        p
    }
}

/// Build a typed checkout response for PayFast custom integration.
///
/// This injects `merchant_id` and `merchant_key` from [`PayFastConfig`], then
/// generates `signature` using the checkout signature algorithm.
pub fn build_checkout(
    cfg: &PayFastConfig,
    sandbox: bool,
    req: CheckoutRequest,
    order: Option<CheckoutFieldOrder>,
) -> CheckoutResponse {
    let mut params = req.into_params();
    if let Some(id) = &cfg.merchant_id {
        params
            .entry("merchant_id".into())
            .or_insert_with(|| id.clone());
    }
    if let Some(key) = &cfg.merchant_key {
        params
            .entry("merchant_key".into())
            .or_insert_with(|| key.clone());
    }

    let order = order.unwrap_or_default();
    let sig = generate_checkout_signature(&params, cfg.passphrase.as_deref(), &order);
    params.insert("signature".into(), sig);

    let url = if sandbox {
        "https://sandbox.payfast.co.za/eng/process"
    } else {
        "https://www.payfast.co.za/eng/process"
    }
    .to_string();

    CheckoutResponse { url, params }
}

/// Build a custom-integration checkout response from arbitrary parameters.
///
/// This helper is useful when you want full control over the fields you send
/// to PayFast (for example, when mirroring the examples in the official
/// custom integration docs or adding advanced options that are not modelled
/// by [`OnceOffPaymentRequest`]).
///
/// - `params` should include all PayFast fields **except** `signature`.
/// - `cfg.merchant_id` / `cfg.merchant_key` are injected if missing.
/// - The signature is generated using [`generate_checkout_signature`].
pub fn build_custom_checkout(
    cfg: &PayFastConfig,
    sandbox: bool,
    mut params: CheckoutParams,
    order: Option<CheckoutFieldOrder>,
) -> CheckoutResponse {
    if let Some(id) = &cfg.merchant_id {
        params
            .entry("merchant_id".into())
            .or_insert_with(|| id.clone());
    }
    if let Some(key) = &cfg.merchant_key {
        params
            .entry("merchant_key".into())
            .or_insert_with(|| key.clone());
    }

    let order = order.unwrap_or_default();
    let sig = generate_checkout_signature(&params, cfg.passphrase.as_deref(), &order);
    params.insert("signature".into(), sig);

    let url = if sandbox {
        "https://sandbox.payfast.co.za/eng/process"
    } else {
        "https://www.payfast.co.za/eng/process"
    }
    .to_string();

    CheckoutResponse { url, params }
}

fn build_params_from_once_off(cfg: &PayFastConfig, req: OnceOffPaymentRequest) -> CheckoutParams {
    let mut params: CheckoutParams = BTreeMap::new();

    if let Some(id) = &cfg.merchant_id {
        params.insert("merchant_id".into(), id.clone());
    }
    if let Some(key) = &cfg.merchant_key {
        params.insert("merchant_key".into(), key.clone());
    }

    params.insert("m_payment_id".into(), req.payment_id);
    params.insert("amount".into(), req.amount);
    params.insert("item_name".into(), req.item_name);

    if let Some(desc) = req.item_description {
        params.insert("item_description".into(), desc);
    }

    if let Some(code) = req.currency.or(req.currency_code) {
        params.insert("currency".into(), code);
    }

    if let Some(v) = req.name_first {
        params.insert("name_first".into(), v);
    }
    if let Some(v) = req.name_last {
        params.insert("name_last".into(), v);
    }
    if let Some(v) = req.email_address {
        params.insert("email_address".into(), v);
    }
    if let Some(v) = req.cell_number {
        params.insert("cell_number".into(), v);
    }

    if let Some(v) = req.return_url {
        params.insert("return_url".into(), v);
    }
    if let Some(v) = req.cancel_url {
        params.insert("cancel_url".into(), v);
    }
    if let Some(v) = req.notify_url {
        params.insert("notify_url".into(), v);
    }
    if let Some(v) = req.notify_method {
        params.insert("notify_method".into(), v);
    }
    if let Some(v) = req.fica_id {
        params.insert("fica_id".into(), v);
    }

    if let Some(v) = req.email_confirmation {
        params.insert(
            "email_confirmation".into(),
            if v { "1".to_string() } else { "0".to_string() },
        );
    }
    if let Some(v) = req.confirmation_address {
        params.insert("confirmation_address".into(), v);
    }

    for (k, v) in req.custom {
        params.insert(k, v);
    }

    params
}

fn apply_subscription(params: &mut CheckoutParams, sub: SubscriptionOptions) {
    if let Some(v) = sub.subscription_type {
        params.insert("subscription_type".into(), v);
    }
    if let Some(v) = sub.billing_date {
        params.insert("billing_date".into(), v);
    }
    if let Some(v) = sub.recurring_amount {
        params.insert("recurring_amount".into(), v);
    }
    if let Some(v) = sub.frequency {
        params.insert("frequency".into(), v);
    }
    if let Some(v) = sub.cycles {
        params.insert("cycles".into(), v);
    }

    if let Some(v) = sub.subscription_notify_email {
        params.insert("subscription_notify_email".into(), v.to_string());
    }
    if let Some(v) = sub.subscription_notify_webhook {
        params.insert("subscription_notify_webhook".into(), v.to_string());
    }
    if let Some(v) = sub.subscription_notify_buyer {
        params.insert("subscription_notify_buyer".into(), v.to_string());
    }
}

fn apply_split(params: &mut CheckoutParams, split: SplitPayment) {
    // For now we map these into custom fields; users can adapt encoding as
    // needed for their own reporting.
    if let Some(v) = split.primary_receiver {
        params.insert("custom_str3".into(), v);
    }
    if let Some(v) = split.secondary_receiver {
        params.insert("custom_str4".into(), v);
    }
    if let Some(v) = split.secondary_amount {
        params.insert("custom_str5".into(), v);
    }
    if let Some(v) = split.setup.or_else(|| {
        split
            .setup_payload
            .and_then(|p| serde_json::to_string(&p).ok())
    }) {
        params.insert("setup".into(), v);
    }
    for (k, v) in split.custom {
        params.insert(k, v);
    }
}

/// Build a once-off checkout response ready to POST/redirect to PayFast.
pub fn build_once_off_checkout(
    cfg: &PayFastConfig,
    sandbox: bool,
    req: OnceOffPaymentRequest,
) -> CheckoutResponse {
    let mut params = build_params_from_once_off(cfg, req);
    let order = CheckoutFieldOrder::default();
    let sig = generate_checkout_signature(&params, cfg.passphrase.as_deref(), &order);
    params.insert("signature".into(), sig);

    let url = if sandbox {
        "https://sandbox.payfast.co.za/eng/process"
    } else {
        "https://www.payfast.co.za/eng/process"
    }
    .to_string();

    CheckoutResponse { url, params }
}

/// Build a subscription (recurring) checkout response.
pub fn build_subscription_checkout(
    cfg: &PayFastConfig,
    sandbox: bool,
    req: AdvancedPaymentRequest,
) -> CheckoutResponse {
    let mut params = build_params_from_once_off(cfg, req.base);
    apply_subscription(&mut params, req.subscription);
    apply_split(&mut params, req.split);

    let order = CheckoutFieldOrder::default();
    let sig = generate_checkout_signature(&params, cfg.passphrase.as_deref(), &order);
    params.insert("signature".into(), sig);

    let url = if sandbox {
        "https://sandbox.payfast.co.za/eng/process"
    } else {
        "https://www.payfast.co.za/eng/process"
    }
    .to_string();

    CheckoutResponse { url, params }
}
