//! # wangapayfast-rs
//!
//! Helpers for working with [PayFast](https://www.payfast.co.za/) payments and
//! ITN (Instant Transaction Notification) messages in Rust services.
//!
//! ---
//!
//! Developed with love by **1nga Solutions** · Logo: <https://www.1nga.com/logo.svg>
//!
//! The main focus of this crate is:
//! - Parsing the `application/x-www-form-urlencoded` ITN body
//! - Re‐generating and verifying the PayFast ITN signature
//! - Strongly-typed access to common ITN fields (amounts, status, method, etc.)
//! - Helper predicates for business notifications (paid / failed / cancelled)
//! - Helpers to generate checkout signatures for custom integrations
//! - Optional HTTP-based post-back verification to PayFast (`http` feature)
//! - Optional API client helpers for subscriptions/refunds/transactions (`api` feature)
//! - Optional onsite payment identifier generation (`onsite` feature)
//!
//! ## Quick start
//!
//! ```no_run
//! use std::collections::BTreeMap;
//!
//! use wangapayfast_rs::{ItnNotification, verify_itn_signature};
//!
//! # fn handler(raw_body: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
//! // 1) Parse the raw `application/x-www-form-urlencoded` ITN body.
//! let itn = ItnNotification::from_body(raw_body)?;
//!
//! // 2) Verify the signature using your PayFast passphrase (if configured).
//! //    Pass `None` if you do not use a passphrase.
//! if !verify_itn_signature(&itn.raw, Some("your-payfast-passphrase")) {
//!     // Invalid / forged ITN – reject
//!     return Err("invalid ITN signature".into());
//! }
//!
//! // 3) Use the parsed fields (amount, status, etc.) from `itn.params()`.
//! let status = itn.payment_status();
//! if status.is_complete() && itn.is_gross_amount("100.00") {
//!     // mark order as paid, trigger your own notifications, etc.
//! }
//! # Ok(())
//! # }
//! ```
//!
//! The crate follows the signature algorithm described in the PayFast
//! documentation for ITN:
//!
//! 1. Start from all form fields received in the ITN (key–value pairs).
//! 2. Remove the `signature` field.
//! 3. Remove any fields whose value is blank after trimming.
//! 4. Sort the remaining fields by key (lexicographically).
//! 5. URL‑encode each value in the same way as an HTML form
//!    (`application/x-www-form-urlencoded`, spaces become `+`).
//! 6. Join as a query string: `key=value&key2=value2...`.
//! 7. If you use a passphrase, append `&passphrase=your-passphrase` and hash
//!    the result.
//! 8. Compute the MD5 hex digest – that is the expected signature.

#![warn(missing_docs)]

#[cfg(feature = "api")]
mod api;
mod error;
mod itn;
mod onsite;
mod payments;

pub use crate::error::{Error, Result};
pub use crate::itn::{
    generate_checkout_signature, generate_itn_signature, try_generate_checkout_signature,
    verify_itn_signature, CheckoutFieldOrder, CheckoutParams, ItnNotification, ItnPaymentStatus,
    ItnRequest, PayFastConfig, PaymentMethod,
};
pub use crate::payments::{
    build_checkout, build_custom_checkout, build_once_off_checkout, build_subscription_checkout,
    try_build_checkout, try_build_custom_checkout, AdvancedPaymentRequest, CheckoutRequest,
    CheckoutResponse, OnceOffPaymentRequest, SplitPayment, SplitPaymentRule, SplitPaymentSetup,
    SubscriptionOptions,
};

#[cfg(feature = "http")]
pub use crate::itn::{post_back_validate_itn, ItnValidationOutcome};

#[cfg(feature = "api")]
pub use crate::api::{
    generate_api_signature, AdhocCharge, ApiEnvironment, PayFastApiClient, PayFastApiConfig,
    RefundCreate, SubscriptionUpdate, TransactionHistoryByDate, TransactionHistoryByMonth,
    TransactionHistoryRange,
};

pub use crate::onsite::{card_update_url, OnsiteEnvironment};

#[cfg(feature = "onsite")]
pub use crate::onsite::generate_payment_identifier;
