macro_rules! _match_role {
    ($role:ident, $fn:tt) => {
        #[allow(clippy::redundant_closure_call)]
        match $role {
            RoleKind::Alice => {
                #[allow(dead_code)]
                type Role = Alice<AL, BL, AA, BA>;
                $fn()
            }
            RoleKind::Bob => {
                #[allow(dead_code)]
                type Role = Bob<AL, BL, AA, BA>;
                $fn()
            }
        }
    };
}

#[macro_export]
macro_rules! with_swap_types {
    ($metadata:expr, $fn:tt) => {{
        use bitcoin_support::BitcoinQuantity;
        use ethereum_support::EtherQuantity;
        let metadata = $metadata;

        use crate::swap_protocols::{asset::Assets, Ledgers};

        match metadata {
            Metadata {
                alpha_ledger: Ledgers::Bitcoin(_),
                beta_ledger: Ledgers::Ethereum(_),
                alpha_asset: Assets::Bitcoin(_),
                beta_asset: Assets::Ether(_),
                role,
            } => {
                #[allow(dead_code)]
                type AL = Bitcoin;
                #[allow(dead_code)]
                type BL = Ethereum;
                #[allow(dead_code)]
                type AA = BitcoinQuantity;
                #[allow(dead_code)]
                type BA = EtherQuantity;

                _match_role!(role, $fn)
            }
            Metadata {
                alpha_ledger: Ledgers::Bitcoin(_),
                beta_ledger: Ledgers::Ethereum(_),
                alpha_asset: Assets::Bitcoin(_),
                beta_asset: Assets::Erc20(_),
                role,
            } => {
                #[allow(dead_code)]
                type AL = Bitcoin;
                #[allow(dead_code)]
                type BL = Ethereum;
                #[allow(dead_code)]
                type AA = BitcoinQuantity;
                #[allow(dead_code)]
                type BA = Erc20Quantity;

                _match_role!(role, $fn)
            }
            Metadata {
                alpha_ledger: Ledgers::Ethereum(_),
                beta_ledger: Ledgers::Bitcoin(_),
                alpha_asset: Assets::Ether(_),
                beta_asset: Assets::Bitcoin(_),
                role,
            } => {
                #[allow(dead_code)]
                type AL = Ethereum;
                #[allow(dead_code)]
                type BL = Bitcoin;
                #[allow(dead_code)]
                type AA = EtherQuantity;
                #[allow(dead_code)]
                type BA = BitcoinQuantity;

                _match_role!(role, $fn)
            }
            Metadata {
                alpha_ledger: Ledgers::Ethereum(_),
                beta_ledger: Ledgers::Bitcoin(_),
                alpha_asset: Assets::Erc20(_),
                beta_asset: Assets::Bitcoin(_),
                role,
            } => {
                #[allow(dead_code)]
                type AL = Ethereum;
                #[allow(dead_code)]
                type BL = Bitcoin;
                #[allow(dead_code)]
                type AA = Erc20Quantity;
                #[allow(dead_code)]
                type BA = BitcoinQuantity;

                _match_role!(role, $fn)
            }
            _ => unimplemented!(),
        }
    }};
}

macro_rules! _match_role_bob {
    ($role:ident, $fn:tt) => {
        #[allow(clippy::redundant_closure_call)]
        match $role {
            RoleKind::Bob => {
                #[allow(dead_code)]
                type Role = Bob<AL, BL, AA, BA>;
                $fn()
            }
            _ => Err(HttpApiProblem::with_title_and_type_from_status(400)
                .set_detail("Requested action is not supported for this role")),
        }
    };
}

#[macro_export]
macro_rules! with_swap_types_bob {
    ($metadata:expr, $fn:tt) => {{
        use crate::swap_protocols::{asset::Assets, Ledgers};
        use bitcoin_support::BitcoinQuantity;
        use ethereum_support::EtherQuantity;
        let metadata = $metadata;

        match metadata {
            Metadata {
                alpha_ledger: Ledgers::Bitcoin(_),
                beta_ledger: Ledgers::Ethereum(_),
                alpha_asset: Assets::Bitcoin(_),
                beta_asset: Assets::Ether(_),
                role,
            } => {
                #[allow(dead_code)]
                type AL = Bitcoin;
                #[allow(dead_code)]
                type BL = Ethereum;
                #[allow(dead_code)]
                type AA = BitcoinQuantity;
                #[allow(dead_code)]
                type BA = EtherQuantity;

                _match_role_bob!(role, $fn)
            }
            Metadata {
                alpha_ledger: Ledgers::Bitcoin(_),
                beta_ledger: Ledgers::Ethereum(_),
                alpha_asset: Assets::Bitcoin(_),
                beta_asset: Assets::Erc20(_),
                role,
            } => {
                #[allow(dead_code)]
                type AL = Bitcoin;
                #[allow(dead_code)]
                type BL = Ethereum;
                #[allow(dead_code)]
                type AA = BitcoinQuantity;
                #[allow(dead_code)]
                type BA = Erc20Quantity;

                _match_role_bob!(role, $fn)
            }
            Metadata {
                alpha_ledger: Ledgers::Ethereum(_),
                beta_ledger: Ledgers::Bitcoin(_),
                alpha_asset: Assets::Ether(_),
                beta_asset: Assets::Bitcoin(_),
                role,
            } => {
                #[allow(dead_code)]
                type AL = Ethereum;
                #[allow(dead_code)]
                type BL = Bitcoin;
                #[allow(dead_code)]
                type AA = EtherQuantity;
                #[allow(dead_code)]
                type BA = BitcoinQuantity;

                _match_role_bob!(role, $fn)
            }
            Metadata {
                alpha_ledger: Ledgers::Ethereum(_),
                beta_ledger: Ledgers::Bitcoin(_),
                alpha_asset: Assets::Erc20(_),
                beta_asset: Assets::Bitcoin(_),
                role,
            } => {
                #[allow(dead_code)]
                type AL = Ethereum;
                #[allow(dead_code)]
                type BL = Bitcoin;
                #[allow(dead_code)]
                type AA = Erc20Quantity;
                #[allow(dead_code)]
                type BA = BitcoinQuantity;

                _match_role_bob!(role, $fn)
            }
            _ => unimplemented!(),
        }
    }};
}
