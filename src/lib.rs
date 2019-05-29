#![feature(async_await)]

use {
    chrono::prelude::*,
    derivative::Derivative,
    failure::Fallible,
    futures::compat::*,
    http::Method,
    reqwest::r#async::Client as HttpClient,
    serde::{Deserialize, Serialize},
    std::{collections::HashMap, fmt::Display, ops::Deref},
};

pub mod models;

const BASE: &str = "https://api.hitbtc.com";

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct Client {
    #[derivative(Debug = "ignore")]
    http_client: HttpClient,
}

impl Client {
    pub fn new() -> Self {
        Self {
            http_client: HttpClient::new(),
        }
    }

    async fn request<'a, T, Q, U>(&'a self, method: Method, url: U, query_params: Q) -> Fallible<T>
    where
        T: for<'de> Deserialize<'de>,
        Q: Serialize,
        U: Display,
    {
        Ok(self
            .http_client
            .clone()
            .request(method, &format!("{}{}", BASE, &url.to_string()))
            .query(&query_params)
            .send()
            .compat()
            .await?
            .json()
            .compat()
            .await?)
    }

    pub async fn get_currencies(&self) -> Fallible<Vec<models::CurrencyInfo>> {
        let method = Method::GET;
        let url = "/api/2/public/currency";

        Ok(self.request(method, url, ()).await?)
    }

    pub fn login(self, api_key: String, api_secret: String) -> AuthenticatedClient {
        AuthenticatedClient {
            public: self,
            api_key,
            api_secret,
        }
    }
}

impl Deref for AuthenticatedClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.public
    }
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct AuthenticatedClient {
    public: Client,
    api_key: String,
    api_secret: String,
}

impl AuthenticatedClient {
    pub fn logout(self) -> Client {
        self.public
    }

    async fn request<'a, T, Q, U>(&'a self, method: Method, url: U, query_params: Q) -> Fallible<T>
    where
        T: for<'de> Deserialize<'de>,
        Q: Serialize,
        U: Display,
    {
        Ok(self
            .http_client
            .clone()
            .request(method, &format!("{}{}", BASE, &url.to_string()))
            .basic_auth(&self.api_key, Some(&self.api_secret))
            .query(&query_params)
            .send()
            .compat()
            .await?
            .json()
            .compat()
            .await?)
    }

    pub async fn get_deposit_address(
        &self,
        currency: String,
    ) -> Fallible<models::DepositAddressInfo> {
        let method = Method::POST;

        let url = format!("/api/2/account/crypto/address/{}", currency);

        Ok(self.request(method, url, ()).await?)
    }

    pub async fn get_transactions_history(
        &self,
        transaction_id: Option<String>,
        from: Option<DateTime<Utc>>,
        till: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Fallible<Vec<models::Transaction>> {
        let method = Method::GET;

        let mut url = "/api/v2/account/transactions".to_string();
        if let Some(id) = transaction_id {
            url.push_str(&format!("/{}", id))
        }

        let query_params = std::iter::empty()
            .chain(from.map(|v| ("from", v.to_rfc3339())))
            .chain(till.map(|v| ("till", v.to_rfc3339())))
            .chain(limit.map(|v| ("limit", v.to_string())))
            .collect::<HashMap<_, _>>();

        Ok(self.request(method, url, query_params).await?)
    }
}
