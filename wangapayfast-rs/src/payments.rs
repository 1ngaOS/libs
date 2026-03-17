use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{generate_checkout_signature, CheckoutFieldOrder, CheckoutParams, PayFastConfig};

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
    if let Some(v) = split.setup {
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
