use std::{sync::Arc, time::Duration};
use fmp_client::{FmpClient, FmpError, Quote};
use moka::future::Cache;
use crate::{error::AppError, symbol::Symbol};

pub struct MarketService {
    fmp: FmpClient,
    quotes: Cache<Symbol, Quote>,
}

impl MarketService {
    pub fn new(fmp: FmpClient) -> Self {
        Self {
            fmp,
            quotes: Cache::builder()
                .max_capacity(5_000)
                .time_to_live(Duration::from_secs(10))
                .build(),
        }
    }

    pub async fn quote(&self, symbol: &Symbol) -> Result<Quote, AppError> {
        let fmp = self.fmp.clone();
        let sym = symbol.clone();

        self.quotes
            .try_get_with(symbol.clone(), async move { fmp.quote(sym.as_str()).await })
            .await
            .map_err(|e: Arc<FmpError>| e.into())
    }
}