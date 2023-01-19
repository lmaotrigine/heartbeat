use super::Snowflake;
use base64::prelude::*;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

pub fn generate_token(id: Snowflake) -> String {
    let engine = BASE64_STANDARD_NO_PAD;
    let now = chrono::Utc::now();
    let enc_now = engine.encode(now.timestamp().to_be_bytes());
    let enc_id = engine.encode(id.id().to_string());
    let mut rng = thread_rng();
    let random_string = std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(27)
        .collect::<String>();
    format!("{}.{}.{}", enc_id, enc_now, random_string)
}
