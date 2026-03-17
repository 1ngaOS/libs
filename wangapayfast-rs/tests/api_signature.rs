use std::collections::BTreeMap;

use wangapayfast_rs::generate_api_signature;

#[test]
fn api_signature_matches_known_md5() {
    // This expected hash is computed using the official PayFast PHP SDK
    // algorithm (sorted keys, form-urlencode values, append passphrase).
    let mut data = BTreeMap::new();
    data.insert("merchant-id".to_string(), "10018867".to_string());
    data.insert("version".to_string(), "v1".to_string());
    data.insert(
        "timestamp".to_string(),
        "2020-08-07T12:34:56+00:00".to_string(),
    );
    data.insert("from".to_string(), "2020-08-01".to_string());
    data.insert("to".to_string(), "2020-08-07".to_string());
    data.insert("limit".to_string(), "1000".to_string());
    data.insert("offset".to_string(), "0".to_string());
    data.insert("testing".to_string(), "true".to_string());

    let sig = generate_api_signature(&data, Some("2uU_k5q_vRS_"));
    assert_eq!(sig, "faf7ec5da63081844a163fc913a4d2a4");
}
