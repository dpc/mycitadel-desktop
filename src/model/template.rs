// MyCitadel desktop wallet: bitcoin & RGB wallet based on GTK framework.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@pandoraprime.ch>
//
// Copyright (C) 2022 by Pandora Prime Sarl, Switzerland.
//
// This software is distributed without any warranty. You should have received
// a copy of the AGPL-3.0 License along with this software. If not, see
// <https://www.gnu.org/licenses/agpl-3.0-standalone.html>.

use chrono::prelude::*;

use super::{Bip43, PublicNetwork, SpendingCondition, WalletFormat};
use crate::model::{SigsReq, TimelockReq};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Requirement {
    Allow,
    Require,
    Deny,
}

impl Default for Requirement {
    fn default() -> Self {
        Requirement::Allow
    }
}

/// Wallet template is a way to define constrained version of a wallet descriptor, but unlike
/// [`super::WalletDescriptor`] not having restrains on the internal consistency between amount of
/// signatures already present and condition parameters.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct WalletTemplate {
    pub format: WalletFormat,
    pub min_signer_count: Option<u16>,
    pub max_signer_count: Option<u16>,
    pub hardware_req: Requirement,
    pub watch_only_req: Requirement,
    pub conditions: Vec<SpendingCondition>,
    pub network: PublicNetwork,
}

impl WalletTemplate {
    pub fn singlesig(
        taproot: bool,
        network: PublicNetwork,
        require_hardware: bool,
    ) -> WalletTemplate {
        let format = if taproot {
            Bip43::singlesig_segwit0()
        } else {
            Bip43::singlelsig_taproot()
        };
        let hardware_req = match require_hardware {
            true => Requirement::Require,
            false => Requirement::Deny,
        };
        let watch_only_req = match require_hardware {
            true => Requirement::Deny,
            false => Requirement::Require,
        };
        WalletTemplate {
            format: format.into(),
            min_signer_count: Some(1),
            max_signer_count: Some(1),
            hardware_req,
            watch_only_req,
            conditions: vec![SpendingCondition::default()],
            network,
        }
    }

    /// # Panics
    ///
    /// If `sigs_required` is less than 3.
    pub fn hodling(
        network: PublicNetwork,
        sigs_required: u16,
        hardware_req: Requirement,
        watch_only_req: Requirement,
    ) -> WalletTemplate {
        let now = Utc::now();
        if sigs_required < 3 {
            unreachable!("WalletTemplate::hodling must require at least 3 signers")
        }
        let conditions = vec![
            SpendingCondition {
                sigs: SigsReq::All,
                timelock: TimelockReq::Anytime,
            },
            SpendingCondition {
                sigs: SigsReq::Any,
                timelock: TimelockReq::AfterTime(now.with_year(now.year() + 5).unwrap()),
            },
        ];
        WalletTemplate {
            format: Bip43::multisig_descriptor().into(),
            min_signer_count: Some(sigs_required),
            max_signer_count: None,
            hardware_req,
            watch_only_req,
            conditions,
            network,
        }
    }

    /// # Panics
    ///
    /// If `sigs_required` is `Some(0)` or `Some(1)`.
    pub fn multisig(
        network: PublicNetwork,
        sigs_required: Option<u16>,
        hardware_req: Requirement,
        watch_only_req: Requirement,
    ) -> WalletTemplate {
        let now = Utc::now();
        let conditions = match sigs_required {
            None => vec![SpendingCondition::default()],
            Some(0) | Some(1) => unreachable!("WalletTemplate::multisig must expect > 1 signature"),
            Some(2) => vec![
                SpendingCondition {
                    sigs: SigsReq::All,
                    timelock: TimelockReq::Anytime,
                },
                SpendingCondition {
                    sigs: SigsReq::Any,
                    timelock: TimelockReq::AfterTime(now.with_year(now.year() + 5).unwrap()),
                },
            ],
            Some(3) => vec![
                SpendingCondition {
                    sigs: SigsReq::AtLeast(2),
                    timelock: TimelockReq::Anytime,
                },
                SpendingCondition {
                    sigs: SigsReq::Any,
                    timelock: TimelockReq::AfterTime(now.with_year(now.year() + 5).unwrap()),
                },
            ],
            Some(count) => vec![
                SpendingCondition {
                    sigs: SigsReq::AtLeast(count - 1),
                    timelock: TimelockReq::Anytime,
                },
                SpendingCondition {
                    sigs: SigsReq::AtLeast(count / 2 + count % 2),
                    timelock: TimelockReq::AfterTime(now.with_year(now.year() + 3).unwrap()),
                },
                SpendingCondition {
                    sigs: SigsReq::Any,
                    timelock: TimelockReq::AfterTime(now.with_year(now.year() + 5).unwrap()),
                },
            ],
        };
        WalletTemplate {
            format: Bip43::multisig_descriptor().into(),
            min_signer_count: sigs_required.or(Some(2)),
            max_signer_count: None,
            hardware_req,
            watch_only_req,
            conditions,
            network,
        }
    }
}