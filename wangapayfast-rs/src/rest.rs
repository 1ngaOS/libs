#![cfg(feature = "rest-api")]

use std::sync::Arc;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

use crate::{
    build_once_off_checkout, build_subscription_checkout, AdvancedPaymentRequest,
    CheckoutResponse, ItnNotification, OnceOffPaymentRequest, PayFastConfig,
    PaymentMethod,
};

/// Shared configuration for the REST API.
#[derive(Clone)]
pub struct RestConfig {
    /// Core PayFast configuration (merchant id/key, passphrase).
    pub payfast: PayFastConfig,
    /// Whether to use the sandbox (`true`) or live gateway (`false`) for
    /// building checkout URLs.
    pub sandbox: bool,
}

/// Build an `axum::Router` exposing ready-to-use PayFast endpoints:
///
/// - `POST /payfast/checkout/once` – once-off payments
/// - `POST /payfast/checkout/subscription` – subscriptions + optional split
/// - `POST /payfast/itn/inspect` – verify + echo ITN data
/// - `GET  /payfast/options` – basic payment method options
pub fn router(cfg: RestConfig) -> Router {
    let state = Arc::new(cfg);

    Router::new()
        .route("/payfast/checkout/once", post(create_once_off))
        .route(
            "/payfast/checkout/subscription",
            post(create_subscription),
        )
        .route("/payfast/itn/inspect", post(itn_inspect))
        .route("/payfast/options", get(payment_options))
        .with_state(state)
}

async fn create_once_off(
    State(cfg): State<Arc<RestConfig>>,
    Json(body): Json<OnceOffPaymentRequest>,
) -> Json<CheckoutResponse> {
    let resp = build_once_off_checkout(&cfg.payfast, cfg.sandbox, body);
    Json(resp)
}

async fn create_subscription(
    State(cfg): State<Arc<RestConfig>>,
    Json(body): Json<AdvancedPaymentRequest>,
) -> Json<CheckoutResponse> {
    let resp = build_subscription_checkout(&cfg.payfast, cfg.sandbox, body);
    Json(resp)
}

#[derive(Serialize)]
struct ItnInspectResponse {
    valid: bool,
    payment_status: String,
    payment_method: String,
    amount_gross: Option<String>,
}

async fn itn_inspect(
    State(_cfg): State<Arc<RestConfig>>,
    body: String,
) -> Json<ItnInspectResponse> {
    // This endpoint intentionally does not enforce signature verification; it is
    // meant as a helper / debugging tool. Consumers should verify signatures
    // in their own ITN handlers using `verify_itn_signature`.
    let notif = match ItnNotification::from_body(body.as_bytes()) {
        Ok(n) => n,
        Err(_) => {
            return Json(ItnInspectResponse {
                valid: false,
                payment_status: "INVALID".into(),
                payment_method: "unknown".into(),
                amount_gross: None,
            })
        }
    };

    let status = notif.payment_status();
    let method = notif.payment_method();
    let amount = notif.params().get("amount_gross").cloned();

    Json(ItnInspectResponse {
        valid: true,
        payment_status: format!("{status:?}"),
        payment_method: format!("{method:?}"),
        amount_gross: amount,
    })
}

#[derive(Serialize)]
struct PaymentOptionsResponse {
    methods: Vec<String>,
}

async fn payment_options() -> Json<PaymentOptionsResponse> {
    let methods = vec![
        format!("{:?}", PaymentMethod::Card),
        format!("{:?}", PaymentMethod::Eft),
        format!("{:?}", PaymentMethod::EftInstant),
    ];
    Json(PaymentOptionsResponse { methods })
}

