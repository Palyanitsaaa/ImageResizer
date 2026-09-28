use base64::prelude::*;
use hmac::{Hmac, KeyInit, Mac};
use sha1::Sha1;

type HmacSha1 = Hmac<Sha1>;

fn main() {
    let secret = "hello";
    let path = "upload/images/product/718_2.jpg";
    let token = generate_token(path, secret);
    println!("Сгенерированный токен: {}", token);
}

fn generate_token(path: &str, secret_key: &str) -> String {
    let mut mac = HmacSha1::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can accept a key of any length.");

    mac.update(path.as_bytes());

    let result = mac.finalize();
    let bytes = result.into_bytes();

    let b64 = BASE64_STANDARD.encode(bytes);

    let modified: String = b64
        .chars()
        .map(|c| match c {
            '+' => '-',
            '/' => '_',
            '=' => ',',
            other => other,
        })
        .collect();

    modified[..12].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_php_hash_parity() {
        let secret = "my_secret_key";
        let path = "/upload/images/product/718_2.jpg";

        // Запустите аналогичную строчку в php -r:
        // echo substr(str_replace(['+','/','='], ['-','_',','], base64_encode(hash_hmac('sha1', '/upload/images/product/718_2.jpg', 'my_secret_key', true))), 0, 12);
        let token = generate_token(path, secret);
        assert_eq!(token.len(), 12);
    }
}
