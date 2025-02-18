use reqwest::Client;
use reqwest::Error;
use reqwest::header::HeaderMap;
use std::collections::HashMap;
use std::sync::OnceLock;

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Self {
        HttpClient {
            client: Client::new(),
        }
    }

    pub async fn get_async(
        &self,
        url: &str,
        params: Option<&HashMap<String, String>>,
    ) -> Result<String, Error> {
        let mut request = self.client.get(url);
        if let Some(query_params) = params {
            request = request.query(query_params);
        }
        let response = request.send().await?;
        response.text().await
    }

    pub async fn post_form_async(
        &self,
        url: &str,
        payload: Option<HashMap<String, String>>,
    ) -> Result<String, Error> {
        let form_data = payload.unwrap_or_default(); // FIXME:
        let response = self.client.post(url).form(&form_data).send().await?;
        response.text().await
    }

    // FIXME: headers
    pub async fn post_binary_async(
        &self,
        url: &str,
        payload: &[u8],
        headers: Option<HeaderMap>,
    ) -> Result<Vec<u8>, Error> {
        let response = self
            .client
            .post(url)
            .headers(headers.unwrap_or_default())
            .body(payload.to_vec())
            .send()
            .await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}
static ASYNC_HTTP_CLIENT: OnceLock<HttpClient> = OnceLock::new();
pub fn client() -> &'static HttpClient {
    ASYNC_HTTP_CLIENT.get_or_init(HttpClient::new)
}
