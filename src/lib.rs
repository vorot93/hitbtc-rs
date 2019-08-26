use {
    bigdecimal::BigDecimal,
    chrono::prelude::*,
    failure::Fallible,
    http::Method,
    hyper_client_util::*,
    log::*,
    maplit::hashmap,
    serde::{Deserialize, Serialize},
    serde_json,
    std::{collections::HashMap, fmt::Display, iter::empty, ops::Deref},
    uuid::Uuid,
};

pub mod models;
use models::*;

const BASE: &str = "https://api.hitbtc.com";

#[derive(Clone, Debug)]
struct HttpClientWrapper(HttpClient);

impl HttpClientWrapper {
    fn with_base_url<P: Display>(&self, path: P) -> String {
        format!("{}{}", BASE, path.to_string())
    }

    async fn request<T: for<'de> Deserialize<'de>, U: Display, Q: Serialize>(
        &self,
        signature: Option<headers::Authorization<headers::authorization::Basic>>,
        method: Method,
        path: U,
        query_params: Q,
    ) -> Fallible<T> {
        let mut req = self
            .0
            .build_request()
            .method(method)
            .uri(&self.with_base_url(format!(
                "{}?{}",
                path.to_string(),
                serde_urlencoded::to_string(query_params)?
            )))?;

        debug!("Sending request: {:?}", req);

        if let Some(signature) = signature {
            req = req.header(signature);
        }

        req.recv_json().await
    }
}

#[derive(Clone, Debug)]
pub struct Client {
    http_client: HttpClientWrapper,
}

impl Client {
    pub fn new(http_client: HttpClient) -> Self {
        Self {
            http_client: HttpClientWrapper(http_client),
        }
    }

    async fn request<T: for<'de> Deserialize<'de>, U: Display, Q: Serialize>(
        &self,
        method: Method,
        path: U,
        query_params: Q,
    ) -> Fallible<T> {
        self.http_client
            .request(None, method, path, query_params)
            .await
    }

    pub async fn get_currencies(&self) -> Fallible<Vec<models::CurrencyInfo>> {
        self.request(Method::GET, "/api/v2/public/currency", ())
            .await
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

#[derive(Clone, Debug)]
pub struct AuthenticatedClient {
    public: Client,
    api_key: String,
    api_secret: String,
}

impl AuthenticatedClient {
    pub fn logout(self) -> Client {
        self.public
    }

    async fn request<T: for<'de> Deserialize<'de>, U: Display, Q: Serialize>(
        &self,
        method: Method,
        path: U,
        query_params: Q,
    ) -> Fallible<T> {
        self.public
            .http_client
            .request(
                Some(headers::Authorization::basic(
                    &self.api_key,
                    &self.api_secret,
                )),
                method,
                path,
                query_params,
            )
            .await
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

        let query_params = empty()
            .chain(from.map(|v| ("from", v.to_rfc3339())))
            .chain(till.map(|v| ("till", v.to_rfc3339())))
            .chain(limit.map(|v| ("limit", v.to_string())))
            .collect::<HashMap<_, _>>();

        Ok(self.request(method, url, query_params).await?)
    }

    pub async fn transfer_account_money(
        &self,
        currency: String,
        amount: BigDecimal,
        direction: AccountMoneyTransferDirection,
    ) -> Fallible<Uuid> {
        let method = Method::POST;

        let url = "/api/v2/account/transfer";

        let query_params = hashmap! {
            "currency" => currency,
            "amount" => amount.to_string(),
            "type" => serde_json::to_string(&direction)?,
        };

        Ok(self.request(method, url, query_params).await?)
    }
}
