use {
    bigdecimal::BigDecimal,
    chrono::prelude::*,
    serde::{Deserialize, Serialize},
    uuid::Uuid,
};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyInfo {
    pub id: String,
    pub full_name: String,
    pub crypto: bool,
    pub payin_enabled: bool,
    pub payin_payment_id: bool,
    pub payin_confirmations: u64,
    pub payout_enabled: bool,
    pub payout_is_payment_id: bool,
    pub payout_fee: BigDecimal,
    pub transfer_enabled: bool,
    pub delisted: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositAddressInfo {
    pub address: String,
    pub payment_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Failed,
    Success,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Payout,
    Payin,
    Deposit,
    Withdraw,
    BankToExchange,
    ExchangeToBank,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub id: Uuid,
    pub index: u64,
    pub currency: String,
    pub amount: BigDecimal,
    pub fee: BigDecimal,
    pub network_fee: BigDecimal,
    pub address: String,
    pub payment_id: Option<String>,
    pub hash: String,
    pub status: TransactionStatus,
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AccountMoneyTransferDirection {
    BankToExchange,
    ExchangeToBank,
}
