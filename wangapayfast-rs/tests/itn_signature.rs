use std::collections::BTreeMap;

use once_cell::sync::Lazy;
use wangapayfast_rs::{
    generate_checkout_signature, generate_itn_signature, CheckoutFieldOrder, CheckoutParams,
    ItnNotification, ItnRequest, PayFastConfig, PaymentMethod, verify_itn_signature,
};

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
    let cfg = PayFastConfig::new(Some(passphrase)).with_merchant("10000100", "46f0cd694581a");

    assert!(verify_itn_signature(&itn, Some(passphrase)));

    let notif = ItnNotification { raw: itn };
    assert!(notif.payment_status().is_complete());
    assert!(notif.is_gross_amount("100.00"));
    assert!(notif.is_expected_merchant(&cfg));
}

#[test]
fn payment_status_and_method_helpers() {
    let mut params = BTreeMap::new();
    params.insert("payment_status".to_string(), "PENDING".to_string());
    params.insert("payment_method".to_string(), "cc".to_string());
    params.insert("amount_gross".to_string(), "50.00".to_string());
    params.insert("merchant_id".to_string(), "mid".to_string());
    params.insert("merchant_key".to_string(), "mkey".to_string());

    let sig = generate_itn_signature(&params, None);
    let mut body = String::new();
    for (k, v) in params.iter() {
        if !body.is_empty() {
            body.push('&');
        }
        body.push_str(&format!(
            "{}={}",
            urlencoding::encode(k),
            urlencoding::encode(v)
        ));
    }
    body.push_str("&signature=");
    body.push_str(&sig);

    let notif = ItnNotification::from_body(body.as_bytes()).expect("parse ITN");
    assert!(notif.payment_status().is_pending());
    assert_eq!(notif.payment_method(), PaymentMethod::Card);
    assert!(notif.is_gross_amount("50.00"));

    let none: Option<String> = None;
    let cfg = PayFastConfig::new(none).with_merchant("mid", "mkey");
    assert!(notif.is_expected_merchant(&cfg));
}

#[test]
fn checkout_signature_respects_order_and_passphrase() {
    let mut params: CheckoutParams = BTreeMap::new();
    params.insert("merchant_id".to_string(), "10000100".to_string());
    params.insert("merchant_key".to_string(), "46f0cd694581a".to_string());
    params.insert("amount".to_string(), "100.00".to_string());
    params.insert("item_name".to_string(), "Order #1234".to_string());

    let order = CheckoutFieldOrder::default();
    let sig1 = generate_checkout_signature(&params, Some("p1"), &order);
    let sig2 = generate_checkout_signature(&params, Some("p2"), &order);
    assert_ne!(sig1, sig2);
}

