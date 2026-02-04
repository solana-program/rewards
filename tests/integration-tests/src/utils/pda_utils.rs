use rewards_program_client::REWARDS_PROGRAM_ID;
use solana_sdk::pubkey::Pubkey;

const VESTING_DISTRIBUTION_SEED: &[u8] = b"vesting_distribution";
const VESTING_RECIPIENT_SEED: &[u8] = b"vesting_recipient";
const EVENT_AUTHORITY_SEED: &[u8] = b"event_authority";

pub fn find_vesting_distribution_pda(mint: &Pubkey, authority: &Pubkey, seeds: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[VESTING_DISTRIBUTION_SEED, mint.as_ref(), authority.as_ref(), seeds.as_ref()],
        &REWARDS_PROGRAM_ID,
    )
}

pub fn find_vesting_recipient_pda(distribution: &Pubkey, recipient: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[VESTING_RECIPIENT_SEED, distribution.as_ref(), recipient.as_ref()],
        &REWARDS_PROGRAM_ID,
    )
}

pub fn find_event_authority_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[EVENT_AUTHORITY_SEED], &REWARDS_PROGRAM_ID)
}
