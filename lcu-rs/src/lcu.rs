use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::{Certificate, Client, Response};
use serde::Serialize;

use crate::auth::Auth;
use crate::error::ApiError;

#[derive(Debug)]
pub struct Lcu {
    client: Client,
    port: String,
}

impl Lcu {
    pub fn new() -> Result<Self, ApiError> {
        let auth = Auth::new()?;
        let mut json_api_headers = HeaderMap::new();
        json_api_headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&auth.basic_auth_token())
                .expect("unable to create header value from auth token"),
        );
        let cert = Certificate::from_pem(include_bytes!("../riotgames.pem"))
            .expect("failed to load certificate");
        let json_api_client = Client::builder()
            .add_root_certificate(cert)
            .default_headers(json_api_headers)
            .build()
            .expect("unable to build client");
        Ok(Self {
            client: json_api_client,
            port: auth.port(),
        })
    }

    fn make_url(&self, endpoint: &str) -> String {
        format!(
            "https://127.0.0.1:{}/{}",
            &self.port,
            endpoint.trim_start_matches('/')
        )
    }

    pub async fn get(&self, endpoint: &str) -> Result<Response, ApiError> {
        Ok(self.client.get(&self.make_url(endpoint)).send().await?)
    }

    pub async fn delete(&self, endpoint: &str) -> Result<Response, ApiError> {
        Ok(self.client.delete(&self.make_url(endpoint)).send().await?)
    }

    pub async fn post<T: Serialize + ?Sized>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<Response, ApiError> {
        Ok(self
            .client
            .post(&self.make_url(endpoint))
            .json(body)
            .send()
            .await?)
    }

    pub async fn put<T: Serialize + ?Sized>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<Response, ApiError> {
        Ok(self
            .client
            .put(&self.make_url(endpoint))
            .json(body)
            .send()
            .await?)
    }

    pub async fn patch<T: Serialize + ?Sized>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<Response, ApiError> {
        Ok(self
            .client
            .patch(&self.make_url(endpoint))
            .json(body)
            .send()
            .await?)
    }
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn lcu_raw_api_get() {
        use super::Lcu;

        let lcu = Lcu::new().unwrap();
        let resp = lcu.get("/lol-summoner/v1/current-summoner").await.unwrap();
        assert_eq!(resp.status(), 200);
        println!("{}", resp.text().await.unwrap());
    }
}
