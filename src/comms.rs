use oxhttp::model::{HeaderName, Method, Request, Status};
use oxhttp::Client;

pub fn send(message: String) -> Option<String> {
    let body = encode(message)?;
    let client = Client::new();
    let mut request = Request::builder(
        Method::POST,
        "https://boundvariable.space/communicate".parse().unwrap(),
    )
    .with_body(body);
    request
        .append_header(
            HeaderName::AUTHORIZATION,
            format!(
                "Bearer {}",
                std::env::var("ICFP2024_TOKEN").expect("Set the ICFP2024_TOKEN env var")
            ),
        )
        .unwrap();
    let response = client.request(request).expect("Failed to send message");
    if response.status() != Status::OK {
        return None;
    }
    let body = response
        .into_body()
        .to_string()
        .expect("Failed to parse the response body");
    Some(body)
}

// TODO: use crate::lexer::unmap_string()
fn encode(message: String) -> Option<Vec<u8>> {
    const MAPPING: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!\"#$%&'()*+,-./:;<=>?@[\\]^_`|~ \n";

    let mut encoded = vec![b'S'];
    for c in message.chars() {
        let idx = MAPPING.find(c)?;
        encoded.push((idx + 33) as u8);
    }
    Some(encoded)
}
