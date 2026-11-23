//! Financial ratios response type for FMP's `/ratios` endpoint.

use serde::{Deserialize, Serialize};

/// Financial ratios for one company over one reporting period.
///
/// Mirrors one element of the array returned by FMP's `/ratios` endpoint,
/// which holds one element per period, newest first. FMP returns about 65
/// fields per period; this struct models the subset the dashboard uses.
/// Unknown fields are ignored during deserialization, so omitted fields
/// are dropped rather than causing errors, and new ones can be added later
/// without changing anything else. As with [`Quote`](crate::types::Quote),
/// the same struct is deserialized from FMP and serialized to API clients,
/// both in camelCase.
///
/// # Units
///
/// Read these conventions before formatting anything for display:
///
/// - **Margins, yields, and payout ratios are fractions, not
///   percentages.** `gross_profit_margin: 0.4878` means 48.78%. Multiply
///   by 100 for display. This is the opposite of
///   [`Quote::change_percentage`](crate::types::Quote::change_percentage),
///   which is already a percentage.
/// - **Valuation and leverage figures are plain multiples.**
///   `price_to_earnings_ratio: 37.85` means 37.85x.
/// - **Per-share figures** are in `reported_currency`.
///
/// # Missing and unreliable values
///
/// Every metric is an `Option<f64>`, so a value FMP omits or sends as
/// `null` becomes `None` here and `null` in the JSON the frontend
/// receives. The frontend should render these as "n/a".
///
/// A present value isn't always meaningful, either. In the AAPL data
/// checked while building this, `interest_coverage_ratio` came back as `0`
/// for recent years and 29 to 40 for earlier ones. That looks like a data
/// gap (no reported interest expense) rather than genuinely zero coverage.
/// Ratios like PEG can also be negative or extreme when earnings are flat
/// or falling. Treat outliers as suspect before charting them.
///
/// # Example
///
/// An abbreviated element, showing the identifying fields and a few
/// metrics:
///
/// ```json
/// {
///   "symbol": "AAPL",
///   "date": "2025-09-27",
///   "fiscalYear": "2025",
///   "period": "FY",
///   "reportedCurrency": "USD",
///   "grossProfitMargin": 0.4690516410716045,
///   "currentRatio": 0.8932929222186667,
///   "priceToEarningsRatio": 34.1068090787717,
///   "debtToEquityRatio": 1.5241072518411023,
///   "dividendYield": 0.004038238951672435
/// }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ratios {
    /// Ticker symbol, e.g. `AAPL`.
    pub symbol: String,
    /// End date of the reporting period, as `YYYY-MM-DD`.
    pub date: String,
    /// Fiscal year the period belongs to. A string, as FMP sends it.
    pub fiscal_year: String,
    /// Period label: `FY` for annual figures, or a quarter such as `Q1`.
    pub period: String,
    /// Currency the company reports in, e.g. `USD`.
    pub reported_currency: String,

    // --- Profitability (fractions) ---
    /// Gross profit divided by revenue.
    pub gross_profit_margin: Option<f64>,
    /// Operating income divided by revenue.
    pub operating_profit_margin: Option<f64>,
    /// Net income divided by revenue.
    pub net_profit_margin: Option<f64>,
    /// EBITDA divided by revenue.
    pub ebitda_margin: Option<f64>,

    // --- Liquidity (multiples) ---
    /// Current assets divided by current liabilities. Below `1.0` means
    /// short-term obligations exceed short-term assets.
    pub current_ratio: Option<f64>,
    /// Like `current_ratio`, but excluding inventory.
    pub quick_ratio: Option<f64>,
    /// Cash and equivalents divided by current liabilities.
    pub cash_ratio: Option<f64>,

    // --- Valuation (multiples) ---
    /// Share price divided by earnings per share.
    pub price_to_earnings_ratio: Option<f64>,
    /// Share price divided by book value per share.
    pub price_to_book_ratio: Option<f64>,
    /// Market capitalization divided by revenue.
    pub price_to_sales_ratio: Option<f64>,
    /// Market capitalization divided by free cash flow.
    pub price_to_free_cash_flow_ratio: Option<f64>,
    /// Enterprise value divided by EBITDA.
    pub enterprise_value_multiple: Option<f64>,

    // --- Leverage (multiples) ---
    /// Total debt divided by shareholders' equity.
    pub debt_to_equity_ratio: Option<f64>,
    /// Total debt divided by total assets.
    pub debt_to_assets_ratio: Option<f64>,
    /// Operating income divided by interest expense. See the note above
    /// about unreliable zeros.
    pub interest_coverage_ratio: Option<f64>,

    // --- Dividends and per-share ---
    /// Annual dividends per share divided by share price, as a fraction
    /// (`0.0031` = 0.31%). FMP also sends a pre-multiplied
    /// `dividendYieldPercentage` field, which this struct does not model.
    pub dividend_yield: Option<f64>,
    /// Dividends paid divided by net income, as a fraction.
    pub dividend_payout_ratio: Option<f64>,
    /// Revenue per share, in `reported_currency`.
    pub revenue_per_share: Option<f64>,
    /// Net income per share, in `reported_currency`.
    pub net_income_per_share: Option<f64>,
    /// Free cash flow per share, in `reported_currency`.
    pub free_cash_flow_per_share: Option<f64>,
}