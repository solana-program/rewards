use pinocchio::{account::AccountView, entrypoint, error::ProgramError, Address, ProgramResult};

use crate::{
    instructions::{
        continuous::{
            claim::process_claim_continuous, close_pool::process_close_reward_pool,
            create_pool::process_create_reward_pool, distribute_reward::process_distribute_reward,
            opt_in::process_opt_in, opt_out::process_opt_out, revoke_user::process_revoke_user,
            set_balance::process_set_balance, sync_balance::process_sync_balance,
        },
        direct::{
            add_recipient::process_add_direct_recipient, claim::process_claim_direct,
            close_distribution::process_close_direct_distribution, close_recipient::process_close_direct_recipient,
            create_distribution::process_create_direct_distribution, revoke_recipient::process_revoke_direct_recipient,
        },
        emit_event::process_emit_event,
        merkle::{
            claim::process_claim_merkle, close_claim::process_close_merkle_claim,
            close_distribution::process_close_merkle_distribution,
            create_distribution::process_create_merkle_distribution, revoke_claim::process_revoke_merkle_claim,
        },
    },
    traits::RewardsInstructionDiscriminators,
};

entrypoint!(process_instruction);

pub fn process_instruction(program_id: &Address, accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    let (discriminator, instruction_data) =
        instruction_data.split_first().ok_or(ProgramError::InvalidInstructionData)?;

    let ix_discriminator = RewardsInstructionDiscriminators::try_from(*discriminator)?;

    match ix_discriminator {
        RewardsInstructionDiscriminators::CreateDirectDistribution => {
            process_create_direct_distribution(program_id, accounts, instruction_data)
        }
        RewardsInstructionDiscriminators::AddDirectRecipient => {
            process_add_direct_recipient(program_id, accounts, instruction_data)
        }
        RewardsInstructionDiscriminators::ClaimDirect => process_claim_direct(program_id, accounts, instruction_data),
        RewardsInstructionDiscriminators::CloseDirectDistribution => {
            process_close_direct_distribution(program_id, accounts, instruction_data)
        }
        RewardsInstructionDiscriminators::CloseDirectRecipient => {
            process_close_direct_recipient(program_id, accounts, instruction_data)
        }
        RewardsInstructionDiscriminators::RevokeDirectRecipient => {
            process_revoke_direct_recipient(program_id, accounts, instruction_data)
        }
        RewardsInstructionDiscriminators::CreateMerkleDistribution => {
            process_create_merkle_distribution(program_id, accounts, instruction_data)
        }
        RewardsInstructionDiscriminators::ClaimMerkle => process_claim_merkle(program_id, accounts, instruction_data),
        RewardsInstructionDiscriminators::CloseMerkleClaim => {
            process_close_merkle_claim(program_id, accounts, instruction_data)
        }
        RewardsInstructionDiscriminators::CloseMerkleDistribution => {
            process_close_merkle_distribution(program_id, accounts, instruction_data)
        }
        RewardsInstructionDiscriminators::RevokeMerkleClaim => {
            process_revoke_merkle_claim(program_id, accounts, instruction_data)
        }
        RewardsInstructionDiscriminators::CreateRewardPool => {
            process_create_reward_pool(program_id, accounts, instruction_data)
        }
        RewardsInstructionDiscriminators::OptIn => process_opt_in(program_id, accounts, instruction_data),
        RewardsInstructionDiscriminators::OptOut => process_opt_out(program_id, accounts, instruction_data),
        RewardsInstructionDiscriminators::DistributeReward => {
            process_distribute_reward(program_id, accounts, instruction_data)
        }
        RewardsInstructionDiscriminators::ClaimContinuous => {
            process_claim_continuous(program_id, accounts, instruction_data)
        }
        RewardsInstructionDiscriminators::SyncBalance => process_sync_balance(program_id, accounts, instruction_data),
        RewardsInstructionDiscriminators::SetBalance => process_set_balance(program_id, accounts, instruction_data),
        RewardsInstructionDiscriminators::CloseRewardPool => {
            process_close_reward_pool(program_id, accounts, instruction_data)
        }
        RewardsInstructionDiscriminators::RevokeUser => process_revoke_user(program_id, accounts, instruction_data),
        RewardsInstructionDiscriminators::EmitEvent => process_emit_event(program_id, accounts),
    }
}
