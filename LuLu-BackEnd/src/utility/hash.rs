use ring::digest::{Context, SHA256};
use hex;

pub struct KeyChipper {
    pub private_key: String,
    pub public_key: String,
}

pub async fn encode_256(text: &str) -> String {
    dotenvy::dotenv().ok();
    let secret = std::env::var("SECRET_KEY").expect("SECRET_KEY should be set").leak();

    let mut ctx = Context::new(&SHA256);
    ctx.update(text.as_bytes());
    ctx.update(secret.as_bytes());
    let diggest = ctx.finish();
    hex::encode(diggest.as_ref())
}

pub async fn create_key(text: &str) -> KeyChipper {
    let pri_key = encode_256(text).await;

    KeyChipper {
        private_key: pri_key.clone(),
        public_key: encode_256(&pri_key).await,
    }
}

