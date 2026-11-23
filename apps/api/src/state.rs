use std::sync::Arc;
use crate::services::{fundamentals::FundamentalsService, market::MarketService};

#[derive(Clone)]
pub struct AppState {
    pub market: Arc<MarketService>,
    pub fundamentals: Arc<FundamentalsService>,
}