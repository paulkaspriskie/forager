use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ratios {
    pub symbol: String,
    pub date: String,
    pub fiscal_year: String,
    pub period: String,
    pub reported_currency: String,

    // profitability
    pub gross_profit_margin: Option<f64>,
    pub operating_profit_margin: Option<f64>,
    pub net_profit_margin: Option<f64>,
    pub ebitda_margin: Option<f64>,

    // liquidity
    pub current_ratio: Option<f64>,
    pub quick_ratio: Option<f64>,
    pub cash_ratio: Option<f64>,

    // valuation
    pub price_to_earnings_ratio: Option<f64>,
    pub price_to_book_ratio: Option<f64>,
    pub price_to_sales_ratio: Option<f64>,
    pub price_to_free_cash_flow_ratio: Option<f64>,
    pub enterprise_value_multiple: Option<f64>,

    // leverage
    pub debt_to_equity_ratio: Option<f64>,
    pub debt_to_assets_ratio: Option<f64>,
    pub interest_coverage_ratio: Option<f64>,

    // dividends & per-share
    pub dividend_yield: Option<f64>,
    pub dividend_payout_ratio: Option<f64>,
    pub revenue_per_share: Option<f64>,
    pub net_income_per_share: Option<f64>,
    pub free_cash_flow_per_share: Option<f64>,
}