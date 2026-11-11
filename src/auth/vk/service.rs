use isahc::{ AsyncReadResponseExt };
use serde_json::Value;
use crate::errors::Error;
use crate::conf::CONF;
use crate::auth::auth::service;
use crate::auth::auth::models::Session;


pub async fn get_email_by_code(code: &String) -> Result<String, Error> {
    let url = format!("{}&code={code}", CONF.read().await.vk_auth_url);
    email_from_url(&url).await
}

async fn email_from_url(url: &str) -> Result<String, Error> {
    let mut response = isahc::get_async(url).await.map_err(|_| Error::HTTP)?;
    if !response.status().is_success() {
        return Err(Error::HTTP);
    }
    let body = response.text().await.map_err(|_| Error::HTTP)?;
    let value: Value = serde_json::from_str(&body).map_err(|_| Error::HTTP)?;
    if !value["error"].is_null() {
        return Err(Error::HTTP);
    }
    value["email"].as_str().map(str::to_owned).ok_or(Error::HTTP)
}

pub async fn try_login_by_code(code: &String) -> Result<Session, Error> {
	let email = get_email_by_code(code).await?;
	if let Some(session) = service::login_by_email(&email).await {
		return Ok(session)
	} else {
		return Err(Error::Common)
	}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::thread;
    use std::time::Duration;

    fn response(status: u16, body: &str, truncated: bool) -> Result<String, Error> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/?code=test-code", listener.local_addr().unwrap());
        let body = body.to_owned();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut reader = BufReader::new(&stream);
            loop {
                let mut line = String::new();
                assert!(reader.read_line(&mut line).unwrap() > 0);
                if line == "\r\n" { break; }
            }
            let length = body.len() + if truncated { 100 } else { 0 };
            write!(stream, "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n{body}").unwrap();
        });
        let result = async_std::task::block_on(email_from_url(&url));
        server.join().unwrap();
        result
    }

    #[test]
    fn vk_email_is_extracted_without_returning_provider_tokens() {
        let email = response(200, r#"{"email":"rider@example.test","access_token":"test-token","user_id":7}"#, false).unwrap();
        assert_eq!(email, "rider@example.test");
    }

    #[test]
    fn provider_and_invalid_responses_return_http_error() {
        for (status, body) in [
            (400, r#"{"error":"invalid_grant"}"#),
            (500, r#"{"email":"rider@example.test"}"#),
            (200, r#"{"error":"invalid_grant","email":"rider@example.test"}"#),
            (200, r#"{"access_token":"test-token"}"#),
            (200, r#"{"email":null}"#),
            (200, "not JSON"),
        ] {
            assert!(matches!(response(status, body, false), Err(Error::HTTP)));
        }
    }

    #[test]
    fn missing_configuration_and_network_failure_return_http_error() {
        let result = async_std::task::block_on(email_from_url("&code=test-code"));
        assert!(matches!(result, Err(Error::HTTP)));
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        drop(listener);
        let result = async_std::task::block_on(email_from_url(&url));
        assert!(matches!(result, Err(Error::HTTP)));
    }

    #[test]
    fn truncated_provider_response_returns_http_error() {
        assert!(matches!(response(200, "{", true), Err(Error::HTTP)));
    }
}
