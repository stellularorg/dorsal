use std::time::{SystemTime, UNIX_EPOCH};

use hex_fmt::HexFmt;
use sha2::{Digest, Sha256};
use uuid::Uuid;

// ids
#[allow(dead_code)]
pub fn uuid() -> String {
    let uuid = Uuid::new_v4();
    return uuid.to_string();
}

#[allow(dead_code)]
pub fn hash(input: String) -> String {
    let mut hasher = <Sha256 as Digest>::new();
    hasher.update(input.into_bytes());

    let res = hasher.finalize();
    return HexFmt(res).to_string();
}

#[allow(dead_code)]
pub fn random_id() -> String {
    return hash(uuid());
}

pub fn unix_epoch_timestamp() -> u128 {
    let right_now = SystemTime::now();
    let time_since = right_now
        .duration_since(UNIX_EPOCH)
        .expect("Time travel is not allowed");

    return time_since.as_millis();
}
