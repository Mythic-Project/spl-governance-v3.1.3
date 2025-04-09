//! General purpose TransactionMessage utility functions

use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::pubkey::Pubkey,
};

/// Unvalidated instruction data, must be treated as untrusted.
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize)]
pub struct TransactionMessage {
    /// The number of signer pubkeys in the account_keys vec.
    pub num_signers: u8,
    /// The number of writable signer pubkeys in the account_keys vec.
    pub num_writable_signers: u8,
    /// The number of writable non-signer pubkeys in the account_keys vec.
    pub num_writable_non_signers: u8,
    /// The list of unique account public keys (including program IDs) that will be used in the provided instructions.
    pub account_keys: Vec<Pubkey>,
    /// The list of instructions to execute.
    pub instructions: Vec<CompiledInstruction>,
    /// List of address table lookups used to load additional accounts
    /// for this transaction.
    pub address_table_lookups: Vec<MessageAddressTableLookup>,
}

/// Concise serialization schema for instructions that make up transaction.
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize)]
pub struct CompiledInstruction {
    /// Indices of the program_id in tx's account_keys
    pub program_id_index: u8,
    /// Indices into the tx's `account_keys` list indicating which accounts to pass to the instruction.
    pub account_indexes: Vec<u8>,
    /// Instruction data.
    pub data: Vec<u8>,
}

/// Address table lookups describe an on-chain address lookup table to use
/// for loading more readonly and writable accounts in a single tx.
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize)]
pub struct MessageAddressTableLookup {
    /// Address lookup table account key
    pub account_key: Pubkey,
    /// List of indexes used to load writable account addresses
    pub writable_indexes: Vec<u8>,
    /// List of indexes used to load readonly account addresses
    pub readonly_indexes: Vec<u8>,
}
