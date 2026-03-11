//! # wangapayfast-rs
//!
//! Helpers for working with [PayFast](https://www.payfast.co.za/) ITN
//! (Instant Transaction Notification) messages in Rust services.
//!
//! The main focus of this crate is:
//! - Parsing the `application/x-www-form-urlencoded` ITN body
//! - Re‐generating the PayFast signature
//! - Verifying that the incoming ITN is authentic
//!
//! This crate intentionally does **not** make outbound HTTP requests. It only
//! deals with the ITN payload you receive in your HTTP handler, so you can
//! integrate it with any web framework.
//!
//! ## Quick start
//!
//! ```no_run
//! use std::collections::BTreeMap;
//!
//! use wangapayfast_rs::{ItnRequest, verify_itn_signature};
//!
//! # fn handler(raw_body: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
//! // 1) Parse the raw `application/x-www-form-urlencoded` ITN body.
//! let itn = ItnRequest::from_body(raw_body)?;
//!
//! // 2) Verify the signature using your PayFast passphrase (if configured).
//! //    Pass `None` if you do not use a passphrase.
//! if !verify_itn_signature(&itn, Some("your-payfast-passphrase")) {
//!     // Invalid / forged ITN – reject
//!     return Err("invalid ITN signature".into());
//! }
//!
//! // 3) Use the parsed fields (amount, status, etc.) from `itn.params()`.
//! let params: &BTreeMap<String, String> = itn.params();
//! let payment_status = params.get("payment_status").cloned().unwrap_or_default();
//! # let _ = payment_status;
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

mod error;
mod itn;

pub use crate::error::{Error, Result};
pub use crate::itn::{
    generate_itn_signature, verify_itn_signature, ItnRequest, PayFastConfig,
};

