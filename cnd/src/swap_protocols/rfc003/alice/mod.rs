mod actions;

pub use self::actions::*;

use crate::{
    asset::Asset,
    seed::SwapSeed,
    swap_protocols::rfc003::{
        ledger::Ledger, ledger_state::LedgerState, messages, ActorState, SwapCommunication,
    },
};
use derivative::Derivative;

#[derive(Clone, Derivative)]
#[derivative(Debug, PartialEq)]
pub struct State<AL: Ledger, BL: Ledger, AA: Asset, BA: Asset> {
    pub swap_communication: SwapCommunication<AL, BL, AA, BA>,
    pub alpha_ledger_state: LedgerState<AL, AA>,
    pub beta_ledger_state: LedgerState<BL, BA>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    pub secret_source: SwapSeed, // Used to derive identities and also to generate the secret.
}

impl<AL: Ledger, BL: Ledger, AA: Asset, BA: Asset> State<AL, BL, AA, BA> {
    pub fn proposed(request: messages::Request<AL, BL, AA, BA>, secret_source: SwapSeed) -> Self {
        Self {
            swap_communication: SwapCommunication::Proposed { request },
            alpha_ledger_state: LedgerState::NotDeployed,
            beta_ledger_state: LedgerState::NotDeployed,
            secret_source,
        }
    }

    pub fn accepted(
        request: messages::Request<AL, BL, AA, BA>,
        response: messages::Accept<AL, BL>,
        secret_source: SwapSeed,
    ) -> Self {
        Self {
            swap_communication: SwapCommunication::Accepted { request, response },
            alpha_ledger_state: LedgerState::NotDeployed,
            beta_ledger_state: LedgerState::NotDeployed,
            secret_source,
        }
    }

    pub fn declined(
        request: messages::Request<AL, BL, AA, BA>,
        response: messages::Decline,
        secret_source: SwapSeed,
    ) -> Self {
        Self {
            swap_communication: SwapCommunication::Declined { request, response },
            alpha_ledger_state: LedgerState::NotDeployed,
            beta_ledger_state: LedgerState::NotDeployed,
            secret_source,
        }
    }

    pub fn request(&self) -> messages::Request<AL, BL, AA, BA> {
        self.swap_communication.request().clone()
    }
}

impl<AL: Ledger, BL: Ledger, AA: Asset, BA: Asset> ActorState for State<AL, BL, AA, BA> {
    type AL = AL;
    type BL = BL;
    type AA = AA;
    type BA = BA;

    fn expected_alpha_asset(&self) -> Self::AA {
        self.swap_communication.request().alpha_asset.clone()
    }

    fn expected_beta_asset(&self) -> Self::BA {
        self.swap_communication.request().beta_asset.clone()
    }

    fn alpha_ledger_mut(&mut self) -> &mut LedgerState<AL, AA> {
        &mut self.alpha_ledger_state
    }

    fn beta_ledger_mut(&mut self) -> &mut LedgerState<BL, BA> {
        &mut self.beta_ledger_state
    }
}
