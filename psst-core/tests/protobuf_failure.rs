use psst_core::error::Error;
use psst_core::protocol::metadata::ActivityPeriod;
use psst_core::util::{deserialize_protobuf, serialize_protobuf};

#[test]
fn deserialize_fails_for_truncated_message() {
    let message = ActivityPeriod {
        start_year: Some(1990),
        end_year: None,
        decade: None,
    };

    let mut encoded = serialize_protobuf(&message).expect("serialization should succeed");
    assert!(!encoded.is_empty(), "encoded message must contain bytes");
    encoded.pop();

    let err = deserialize_protobuf::<ActivityPeriod>(&encoded).expect_err("truncated payload must fail");
    assert!(matches!(err, Error::IoError(_)));
}
