use crate::define_instruction;

use super::continuous::{
    claim::{ClaimContinuousAccounts, ClaimContinuousData},
    close_pool::{CloseRewardPoolAccounts, CloseRewardPoolData},
    create_pool::{CreateRewardPoolAccounts, CreateRewardPoolData},
    distribute_reward::{DistributeRewardAccounts, DistributeRewardData},
    opt_in::{OptInAccounts, OptInData},
    opt_out::{OptOutAccounts, OptOutData},
    revoke_user::{RevokeUserAccounts, RevokeUserData},
    set_balance::{SetBalanceAccounts, SetBalanceData},
    sync_balance::{SyncBalanceAccounts, SyncBalanceData},
};
use super::direct::{
    add_recipient::{AddDirectRecipientAccounts, AddDirectRecipientData},
    claim::{ClaimDirectAccounts, ClaimDirectData},
    close_distribution::{CloseDirectDistributionAccounts, CloseDirectDistributionData},
    close_recipient::{CloseDirectRecipientAccounts, CloseDirectRecipientData},
    create_distribution::{CreateDirectDistributionAccounts, CreateDirectDistributionData},
    revoke_recipient::{RevokeDirectRecipientAccounts, RevokeDirectRecipientData},
};
use super::merkle::{
    claim::{ClaimMerkleAccounts, ClaimMerkleData},
    close_claim::{CloseMerkleClaimAccounts, CloseMerkleClaimData},
    close_distribution::{CloseMerkleDistributionAccounts, CloseMerkleDistributionData},
    create_distribution::{CreateMerkleDistributionAccounts, CreateMerkleDistributionData},
    revoke_claim::{RevokeMerkleClaimAccounts, RevokeMerkleClaimData},
};

// Direct Distribution
define_instruction!(AddDirectRecipient, AddDirectRecipientAccounts, AddDirectRecipientData);
define_instruction!(ClaimDirect, ClaimDirectAccounts, ClaimDirectData);
define_instruction!(CloseDirectDistribution, CloseDirectDistributionAccounts, CloseDirectDistributionData);
define_instruction!(CloseDirectRecipient, CloseDirectRecipientAccounts, CloseDirectRecipientData);
define_instruction!(CreateDirectDistribution, CreateDirectDistributionAccounts, CreateDirectDistributionData);
define_instruction!(RevokeDirectRecipient, RevokeDirectRecipientAccounts, RevokeDirectRecipientData);

// Merkle Distribution
define_instruction!(ClaimMerkle, ClaimMerkleAccounts, ClaimMerkleData);
define_instruction!(CloseMerkleClaim, CloseMerkleClaimAccounts, CloseMerkleClaimData);
define_instruction!(CloseMerkleDistribution, CloseMerkleDistributionAccounts, CloseMerkleDistributionData);
define_instruction!(CreateMerkleDistribution, CreateMerkleDistributionAccounts, CreateMerkleDistributionData);
define_instruction!(RevokeMerkleClaim, RevokeMerkleClaimAccounts, RevokeMerkleClaimData);

// Continuous Distribution
define_instruction!(CreateRewardPool, CreateRewardPoolAccounts, CreateRewardPoolData);
define_instruction!(OptIn, OptInAccounts, OptInData);
define_instruction!(OptOut, OptOutAccounts, OptOutData);
define_instruction!(DistributeReward, DistributeRewardAccounts, DistributeRewardData);
define_instruction!(ClaimContinuous, ClaimContinuousAccounts, ClaimContinuousData);
define_instruction!(SyncBalance, SyncBalanceAccounts, SyncBalanceData);
define_instruction!(SetBalance, SetBalanceAccounts, SetBalanceData);
define_instruction!(CloseRewardPool, CloseRewardPoolAccounts, CloseRewardPoolData);
define_instruction!(RevokeUser, RevokeUserAccounts, RevokeUserData);
