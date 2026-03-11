use std::collections::BTreeMap;

use once_cell::sync::Lazy;
use wangapayfast_rs::{generate_itn_signature, ItnRequest, PayFastConfig, verify_itn_signature};

static SAMPLE_PARAMS: Lazy<BTreeMap<String, String>> = Lazy::new(|| {
    let mut m = BTreeMap::new();
    m.insert("merchant_id".to_string(), "10000100".to_string());
    m.insert("merchant_key".to_string(), "46f0cd694581a".to_string());
    m.insert("amount_gross".to_string(), "100.00".to_string());
    m.insert("item_name".to_string(), "Order #1234".to_string());
    m.insert("payment_status".to_string(), "COMPLETE".to_string());
    m
});

#[test]
fn generates_deterministic_signature_without_passphrase() {
    let sig1 = generate_itn_signature(&SAMPLE_PARAMS, None);
    let sig2 = generate_itn_signature(&SAMPLE_PARAMS, None);
    assert_eq!(sig1, sig2);
}

#[test]
fn different_passphrase_produces_different_signatures() {
    let sig1 = generate_itn_signature(&SAMPLE_PARAMS, Some("secret-1"));
    let sig2 = generate_itn_signature(&SAMPLE_PARAMS, Some("secret-2"));
    assert_ne!(sig1, sig2);
}

#[test]
fn parses_body_and_verifies_signature_roundtrip() {
    let passphrase = "test-passphrase";

    // Build the canonical params and signature.
    let signature = generate_itn_signature(&SAMPLE_PARAMS, Some(passphrase));

    // Encode into a body as PayFast would send it (including signature).
    let mut encoded = String::new();
    for (k, v) in SAMPLE_PARAMS.iter() {
        if !encoded.is_empty() {
            encoded.push('&');
        }
        encoded.push_str(&format!(
            "{}={}",
            urlencoding::encode(k),
            urlencoding::encode(v)
        ));
    }
    encoded.push_str("&signature=");
    encoded.push_str(&signature);

    let body = encoded.into_bytes();

    let itn = ItnRequest::from_body(&body).expect("parse ITN body");
    let config = PayFastConfig::new(Some(passphrase));

    assert!(itn.is_valid(&config));
    assert!(verify_itn_signature(&itn, Some(passphrase)));
}

