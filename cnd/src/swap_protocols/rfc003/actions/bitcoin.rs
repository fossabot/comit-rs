use crate::{
    asset,
    swap_protocols::{
        actions::bitcoin::{SendToAddress, SpendOutput},
        ledger::Bitcoin,
        rfc003::{
            actions::{FundAction, RedeemAction, RefundAction},
            create_swap::HtlcParams,
            DeriveIdentities, Secret,
        },
    },
};
use bitcoin::{Amount, OutPoint, Transaction};
use blockchain_contracts::bitcoin::{rfc003::bitcoin_htlc::BitcoinHtlc, witness::PrimedInput};

impl FundAction<Bitcoin, asset::Bitcoin> for (Bitcoin, asset::Bitcoin) {
    type FundActionOutput = SendToAddress;

    fn fund_action(htlc_params: HtlcParams<Bitcoin, asset::Bitcoin>) -> Self::FundActionOutput {
        let to = htlc_params.compute_address();

        SendToAddress {
            to,
            amount: htlc_params.asset,
            network: htlc_params.ledger.network,
        }
    }
}

impl RefundAction<Bitcoin, asset::Bitcoin> for (Bitcoin, asset::Bitcoin) {
    type RefundActionOutput = SpendOutput;

    fn refund_action(
        htlc_params: HtlcParams<Bitcoin, asset::Bitcoin>,
        htlc_location: OutPoint,
        secret_source: &dyn DeriveIdentities,
        fund_transaction: &Transaction,
    ) -> Self::RefundActionOutput {
        let htlc = BitcoinHtlc::from(htlc_params);

        SpendOutput {
            output: PrimedInput::new(
                htlc_location,
                Amount::from_sat(fund_transaction.output[htlc_location.vout as usize].value),
                htlc.unlock_after_timeout(&*crate::SECP, secret_source.derive_refund_identity()),
            ),
            network: htlc_params.ledger.network,
        }
    }
}

impl RedeemAction<Bitcoin, asset::Bitcoin> for (Bitcoin, asset::Bitcoin) {
    type RedeemActionOutput = SpendOutput;

    fn redeem_action(
        htlc_params: HtlcParams<Bitcoin, asset::Bitcoin>,
        htlc_location: OutPoint,
        secret_source: &dyn DeriveIdentities,
        secret: Secret,
    ) -> Self::RedeemActionOutput {
        let htlc = BitcoinHtlc::from(htlc_params);

        SpendOutput {
            output: PrimedInput::new(
                htlc_location,
                htlc_params.asset.into(),
                htlc.unlock_with_secret(
                    &*crate::SECP,
                    secret_source.derive_redeem_identity(),
                    secret.into_raw_secret(),
                ),
            ),
            network: htlc_params.ledger.network,
        }
    }
}
