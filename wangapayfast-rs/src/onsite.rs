#[cfg(feature = "onsite")]
use serde::Deserialize;

#[cfg(feature = "onsite")]
use crate::error::{Error, Result};
#[cfg(feature = "onsite")]
use crate::{generate_checkout_signature, CheckoutFieldOrder, CheckoutParams, PayFastConfig};

/// Onsite payments environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnsiteEnvironment {
    /// Production endpoints.
    Live,
    /// Sandbox endpoints.
    Sandbox,
}

impl OnsiteEnvironment {
    #[cfg(feature = "onsite")]
    fn process_url(self) -> &'static str {
        match self {
            OnsiteEnvironment::Live => "https://www.payfast.co.za/onsite/process",
            OnsiteEnvironment::Sandbox => "https://sandbox.payfast.co.za/onsite/process",
        }
    }

    fn recurring_update_base(self) -> &'static str {
        match self {
            OnsiteEnvironment::Live => "https://www.payfast.co.za/eng/recurring/update",
            OnsiteEnvironment::Sandbox => "https://sandbox.payfast.co.za/eng/recurring/update",
        }
    }
}

#[cfg(feature = "onsite")]
#[derive(Debug, Deserialize)]
struct IdentifierResponse {
    uuid: String,
}

/// Generate an onsite payment identifier (UUID) by POSTing the signed checkout
/// payload to PayFast's `/onsite/process` endpoint.
///
/// The `params` should contain the usual custom integration fields **except**
/// `signature`. This function injects the configured merchant fields if
/// missing, generates `signature` using checkout signature rules, then posts
/// the form body.
#[cfg(feature = "onsite")]
pub async fn generate_payment_identifier(
    cfg: &PayFastConfig,
    env: OnsiteEnvironment,
    mut params: CheckoutParams,
    order: Option<CheckoutFieldOrder>,
) -> Result<String> {
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

    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (k, v) in params.iter() {
        serializer.append_pair(k, v);
    }
    let body = serializer.finish();

    let client = reqwest::Client::new();
    let resp = client
        .post(env.process_url())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;
    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        return Err(Error::ApiHttp {
            status: status.as_u16(),
            body: text,
        });
    }

    let parsed: IdentifierResponse = serde_json::from_str(&text)?;
    Ok(parsed.uuid)
}

/// Build a link that lets a buyer update card details for a subscription /
/// tokenization agreement.
///
/// Docs: `GET /eng/recurring/update/{token}?return={return}`
pub fn card_update_url(env: OnsiteEnvironment, token: &str, return_url: Option<&str>) -> String {
    let base = env.recurring_update_base();
    match return_url {
        Some(r) => {
            let mut serializer = url::form_urlencoded::Serializer::new(String::new());
            serializer.append_pair("return", r);
            format!("{base}/{token}?{}", serializer.finish())
        }
        None => format!("{base}/{token}"),
    }
}
