mod filter;
mod matchmaker;
mod mempool;

use anoma::protobuf::types::{Intent, IntentBroadcasterMessage, Tx};
use prost::Message;
use thiserror::Error;
use tokio::sync::mpsc::Receiver;

use matchmaker::Matchmaker;

// TODO split Error and Result type in two, one for Result/Error that can only
// happens localy and the other that can happens locally and in the network
#[derive(Error, Debug)]
pub enum Error {
    #[error("Error while decoding intent: {0}")]
    DecodeError(prost::DecodeError),
    #[error("Error initializing the matchmaker: {0}")]
    MatchmakerInit(matchmaker::Error),
    #[error("Error running the matchmaker: {0}")]
    Matchmaker(matchmaker::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct GossipIntent {
    pub matchmaker: Option<Matchmaker>,
}

impl GossipIntent {
    pub fn new(
        config: &anoma::config::Gossip,
    ) -> Result<(Self, Option<Receiver<Tx>>)> {
        let (matchmaker, matchmaker_event_receiver) = if let Some(matchmaker) =
            &config.matchmaker
        {
            let (matchmaker, matchmaker_event_receiver) =
                Matchmaker::new(&matchmaker).map_err(Error::MatchmakerInit)?;
            (Some(matchmaker), Some(matchmaker_event_receiver))
        } else {
            (None, None)
        };
        Ok((
            Self {
                matchmaker,
            },
            matchmaker_event_receiver,
        ))
    }

    async fn apply_matchmaker(&mut self, intent: Intent) -> Option<Result<bool>> {
        if let Some(matchmaker) = &mut self.matchmaker {
            Some(matchmaker.try_match_intent(&intent).await.map_err(Error::Matchmaker))
        } else {None}
    }

    pub async fn apply_intent(&mut self, intent: Intent) -> Result<bool> {
        self.apply_matchmaker(intent).await;
        Ok(true)

    }

    pub fn parse_raw_msg(
        &mut self,
        data: impl AsRef<[u8]>,
    ) -> Result<IntentBroadcasterMessage> {
        IntentBroadcasterMessage::decode(data.as_ref())
            .map_err(Error::DecodeError)
    }
}
