use anoma::protobuf::types::{Intent, Tx};
use prost::Message;
use tokio::sync::mpsc::Receiver;

use super::matchmaker::Matchmaker;
use super::mempool::Mempool;

#[derive(Debug, Clone)]
pub enum OrderbookError {
    DecodeError(prost::DecodeError),
}

impl std::fmt::Display for OrderbookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DecodeError(prost_error) => write!(f, "{}", prost_error),
        }
    }
}
impl std::error::Error for OrderbookError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DecodeError(prost_error) => Some(prost_error),
        }
    }
}

pub type Result<T> = std::result::Result<T, OrderbookError>;

#[derive(Debug)]
pub struct Orderbook {
    pub mempool: Mempool,
    pub matchmaker: Option<Matchmaker>,
}

impl Orderbook {
    pub fn new(
        matchmaker: Option<String>,
        tx_template: Option<String>,
    ) -> (Self, Option<Receiver<Tx>>) {
        let (matchmaker, matchmaker_event_receiver) =
        // TODO instead matchmaker cli option should be something like Option<(String, String)>
            if matchmaker.is_some() && tx_template.is_some() {
                let matchmaker = matchmaker.unwrap();
                let tx_template = tx_template.unwrap();
                let (matchmaker, matchmaker_event_receiver) =
                    Matchmaker::new(matchmaker, tx_template);
                (Some(matchmaker), Some(matchmaker_event_receiver))
            } else {
                (None, None)
            };

        (
            Self {
                mempool: Mempool::new(),
                matchmaker,
            },
            matchmaker_event_receiver,
        )
    }

    pub async fn apply_intent(&mut self, intent: Intent) -> Result<bool> {
        if let Some(matchmaker) = &mut self.matchmaker {
            matchmaker.try_match_intent(&intent).await;
            let _result = matchmaker.add(intent);
        }
        Ok(true)
    }

    pub async fn apply_raw_intent(&mut self, data: &Vec<u8>) -> Result<bool> {
        let intent =
            Intent::decode(&data[..]).map_err(OrderbookError::DecodeError)?;
        self.apply_intent(intent).await
    }
}
