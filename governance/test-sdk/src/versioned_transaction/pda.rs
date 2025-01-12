use solana_sdk::pubkey::Pubkey;


pub const EPHERMAL_SIGNER_SEED: &[u8] = b"ephemeral_signer";
pub const VERSIONED_TRANSACTION_BUFFER_SEED: &[u8] = b"version_transaction";

pub fn get_ephemeral_signer_pda(
    transaction_proposal: &Pubkey,
    ephemeral_signer_index: u8,
    program_id: &Pubkey,
    transaction_index: u16,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            VERSIONED_TRANSACTION_BUFFER_SEED,
            &transaction_proposal.to_bytes(),
            EPHERMAL_SIGNER_SEED,
            &transaction_index.to_le_bytes(),
            &ephemeral_signer_index.to_le_bytes(),
        ],
        program_id,
    )
}
