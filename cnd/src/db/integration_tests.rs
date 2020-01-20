use crate::{
    asset::{Bitcoin as BitcoinAsset, Erc20, Ether},
    db::{
        load_swaps::LoadAcceptedSwap,
        swap_types::{DetermineTypes, SwapTypes},
        AssetKind, LedgerKind, Retrieve, Save, Sqlite, Swap,
    },
    quickcheck::Quickcheck,
    swap_protocols::{
        ledger::{Bitcoin, Ethereum},
        rfc003::{Accept, Request},
    },
};
use std::path::Path;

macro_rules! db_roundtrip_test {
    ($alpha_ledger:ident, $beta_ledger:ident, $alpha_asset:ident, $beta_asset:ident, $expected_swap_types_fn:expr) => {
        paste::item! {
            #[test]
            #[allow(non_snake_case, clippy::redundant_closure_call)]
            fn [<roundtrip_test_ $alpha_ledger _ $beta_ledger _ $alpha_asset _ $beta_asset>]() {
                fn prop(swap: Quickcheck<Swap>,
                        request: Quickcheck<Request<$alpha_ledger, $beta_ledger, $alpha_asset, $beta_asset>>,
                        accept: Quickcheck<Accept<$alpha_ledger, $beta_ledger>>,
                ) -> anyhow::Result<bool> {

                    // unpack the swap from the generic newtype
                    let Swap { swap_id, role, counterparty } = swap.0;

                    // construct the expected swap types from the function we get passed in order to enrich it with the role
                    let expected_swap_types = ($expected_swap_types_fn)(role);

                    let db = Sqlite::new(&Path::new(":memory:"))?;

                    let saved_swap = Swap {
                        swap_id,
                        role,
                        counterparty
                    };
                    let saved_request = Request {
                        swap_id,
                        ..*request
                    };
                    let saved_accept = Accept {
                        swap_id,
                        ..*accept
                    };

                    let (loaded_swap, loaded_request, loaded_accept, loaded_swap_types) =
                    tokio::runtime::Runtime::new()?.block_on(async {
                        db.save(saved_swap.clone()).await?;
                        db.save(saved_request.clone()).await?;
                        db.save(saved_accept.clone()).await?;

                        let loaded_swap = Retrieve::get(&db, &swap_id).await?;
                        // If the assignment of `_at` works then we have a valid NaiveDateTime.
                        let (loaded_request, loaded_accept, _at) = db.load_accepted_swap(&swap_id).await?;
                        let loaded_swap_types = db.determine_types(&swap_id).await?;

                        anyhow::Result::<_>::Ok((loaded_swap, loaded_request, loaded_accept, loaded_swap_types))
                    })?;

                    Ok(
                        saved_request == loaded_request &&
                            saved_accept == loaded_accept &&
                            saved_swap == loaded_swap &&
                            expected_swap_types == loaded_swap_types
                    )
                }

                quickcheck::quickcheck(prop as fn(
                    Quickcheck<Swap>,
                    Quickcheck<Request<$alpha_ledger, $beta_ledger, $alpha_asset, $beta_asset>>,
                    Quickcheck<Accept<$alpha_ledger, $beta_ledger>>,
                ) -> anyhow::Result<bool>);
            }
        }
    };
}

// db_roundtrip_test!(Bitcoin, Ethereum, BitcoinAsset, Ether, |role| {
//    SwapTypes {
//        alpha_ledger: LedgerKind::Bitcoin,
//        beta_ledger: LedgerKind::Ethereum,
//        alpha_asset: AssetKind::Bitcoin,
//        beta_asset: AssetKind::Ether,
//        role,
//    }
//});
// db_roundtrip_test!(Ethereum, Bitcoin, Ether, BitcoinAsset, |role| {
//    SwapTypes {
//        alpha_ledger: LedgerKind::Ethereum,
//        beta_ledger: LedgerKind::Bitcoin,
//        alpha_asset: AssetKind::Ether,
//        beta_asset: AssetKind::Bitcoin,
//        role,
//    }
//});
// db_roundtrip_test!(Bitcoin, Ethereum, BitcoinAsset, Erc20, |role| {
//    SwapTypes {
//        alpha_ledger: LedgerKind::Bitcoin,
//        beta_ledger: LedgerKind::Ethereum,
//        alpha_asset: AssetKind::Bitcoin,
//        beta_asset: AssetKind::Erc20,
//        role,
//    }
//});
// db_roundtrip_test!(Ethereum, Bitcoin, Erc20, BitcoinAsset, |role| {
//    SwapTypes {
//        alpha_ledger: LedgerKind::Ethereum,
//        beta_ledger: LedgerKind::Bitcoin,
//        alpha_asset: AssetKind::Erc20,
//        beta_asset: AssetKind::Bitcoin,
//        role,
//    }
//});
