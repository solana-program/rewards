use pinocchio::{account::AccountView, entrypoint, error::ProgramError, Address, ProgramResult};

use crate::{
    instructions::{
        add_direct_recipient::process_add_direct_recipient, claim_direct::process_claim_direct,
        claim_merkle::process_claim_merkle, close_direct_distribution::process_close_direct_distribution,
        close_direct_recipient::process_close_direct_recipient, close_merkle_claim::process_close_merkle_claim,
        close_merkle_distribution::process_close_merkle_distribution,
        create_direct_distribution::process_create_direct_distribution,
        create_merkle_distribution::process_create_merkle_distribution, emit_event::process_emit_event,
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
        RewardsInstructionDiscriminators::EmitEvent => process_emit_event(program_id, accounts),
    }
}
