use pinocchio::{account::AccountView, entrypoint, error::ProgramError, Address, ProgramResult};

use crate::{
    instructions::{
        add_vesting_recipient::process_add_vesting_recipient, claim_vesting::process_claim_vesting,
        close_vesting_distribution::process_close_vesting_distribution,
        create_vesting_distribution::process_create_vesting_distribution, emit_event::process_emit_event,
    },
    traits::RewardsInstructionDiscriminators,
};

entrypoint!(process_instruction);

pub fn process_instruction(program_id: &Address, accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    let (discriminator, instruction_data) =
        instruction_data.split_first().ok_or(ProgramError::InvalidInstructionData)?;

    let ix_discriminator = RewardsInstructionDiscriminators::try_from(*discriminator)?;

    match ix_discriminator {
        RewardsInstructionDiscriminators::CreateVestingDistribution => {
            process_create_vesting_distribution(program_id, accounts, instruction_data)
        }
        RewardsInstructionDiscriminators::AddVestingRecipient => {
            process_add_vesting_recipient(program_id, accounts, instruction_data)
        }
        RewardsInstructionDiscriminators::ClaimVesting => process_claim_vesting(program_id, accounts, instruction_data),
        RewardsInstructionDiscriminators::CloseVestingDistribution => {
            process_close_vesting_distribution(program_id, accounts, instruction_data)
        }
        RewardsInstructionDiscriminators::EmitEvent => process_emit_event(program_id, accounts),
    }
}
