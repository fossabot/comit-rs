#[macro_use]
mod transition_save;

pub mod alice;
pub mod bitcoin;
pub mod bob;
pub mod ethereum;
pub mod events;
pub mod ledger_state;
pub mod messages;
pub mod state_machine;
pub mod state_store;

pub mod actions;
mod actor_state;
mod ledger;
mod save_state;
mod secret;
mod secret_source;

pub use self::{
    actor_state::ActorState,
    ledger::Ledger,
    ledger_state::{HtlcState, LedgerState},
    save_state::SaveState,
    secret::{FromErr, Secret, SecretHash},
    secret_source::*,
};

pub use self::messages::{Accept, Decline, Request};

use crate::{swap_protocols::asset::Asset, timestamp::Timestamp};

/// Swap request response as received from peer node acting as Bob.
pub type Response<AL, BL> = Result<Accept<AL, BL>, Decline>;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("btsieve")]
    Btsieve,
    #[error("timer error")]
    TimerError,
    #[error("incorrect funding")]
    IncorrectFunding,
    #[error("internal error: {0}")]
    Internal(String),
}

pub fn alpha_expiry_has_passed<AL: Ledger, BL: Ledger, AA: Asset, BA: Asset>(
    request: &Request<AL, BL, AA, BA>,
) -> bool {
    request.alpha_expiry < Timestamp::now()
}

pub fn beta_expiry_has_passed<AL: Ledger, BL: Ledger, AA: Asset, BA: Asset>(
    request: &Request<AL, BL, AA, BA>,
) -> bool {
    request.beta_expiry < Timestamp::now()
}
