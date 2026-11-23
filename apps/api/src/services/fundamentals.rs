use std::{sync::Arc, time::Duration};
use fmp_client::{FmpClient, FmpError, Ratios};
use moka::future::Cache;
use serde::Deserialize;
use crate::{error::AppError, symbol::Symbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Period {
    Annual,
    Quarter,
}

impl Period {
    fn as_fmp(self) -> &'static str {
        match self {
            Period::Annual => "FY",
            Period::Quarter => "quarter",
        }
    }
}

type RatiosKey = (Symbol, Period, u32);

pub struct FundamentalsService {
    fmp: FmpClient,
    ratios: Cache<RatiosKey, Arc<Vec<Ratios>>>,
}

impl FundamentalsService {
    pub fn new(fmp: FmpClient) -> Self {
        Self {
            fmp,
            ratios: Cache::builder()
                .max_capacity(2_000)
                .time_to_live(Duration::from_secs(6 * 60 * 60))
                .build(),
        }
    }

    pub async fn ratios(
        &self,
        symbol: &Symbol,
        period: Period,
        limit: u32,
    ) -> Result<Arc<Vec<Ratios>>, AppError> {
        let limit = limit.clamp(1, 20);
        let key = (symbol.clone(), period, limit);
        let fmp = self.fmp.clone();
        let sym = symbol.clone();

        self.ratios
            .try_get_with(key, async move {
                fmp.ratios(sym.as_str(), period.as_fmp(), limit).await.map(Arc::new)
            })
            .await
            .map_err(|e: Arc<FmpError>| e.into())
    }
}