#![cfg(feature = "test-sbf")]

mod program_test;

use {
    program_test::*,
    solana_program_test::tokio,
    solana_sdk::{
        hash::hashv,
        instruction::{AccountMeta, Instruction},
        signature::Keypair,
        signer::Signer,
        system_instruction, system_program,
    },
    spl_governance::{
        error::GovernanceError,
        state::{
            enums::{ProposalState, TransactionExecutionStatus},
            native_treasury::get_native_treasury_address,
        },
        tools::{spl_token::inline_spl_token, transaction_message::TransactionMessage},
    },
    versioned_transaction_ext::VaultTransactionMessageExt,
};

#[tokio::test]
async fn test_create_transaction_buffer_and_execute() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_cookie = governance_test.with_realm().await;
    let governed_account_cookie = governance_test.with_governed_account().await;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit(&realm_cookie)
        .await
        .unwrap();

    let mut governance_cookie = governance_test
        .with_governance(
            &realm_cookie,
            &governed_account_cookie,
            &token_owner_record_cookie,
        )
        .await
        .unwrap();

    governance_test
        .with_native_treasury(&governance_cookie)
        .await;

    let mut proposal_cookie = governance_test
        .with_proposal(&token_owner_record_cookie, &mut governance_cookie)
        .await
        .unwrap();

    let signatory_record_cookie = governance_test
        .with_signatory(
            &proposal_cookie,
            &governance_cookie,
            &token_owner_record_cookie,
        )
        .await
        .unwrap();

    let treasury_address =
        get_native_treasury_address(&governance_test.program_id, &governance_cookie.address);

    let mut instructions = Vec::new();

    // Number of times to clone the instruction
    let instruction_count = 60;

    // Base instruction
    let base_instruction =
        system_instruction::transfer(&treasury_address, &Keypair::new().pubkey(), 1000000);

    // Fill the vec with cloned instructions
    for _ in 0..instruction_count {
        instructions.push(base_instruction.clone());
    }
    let transaction_message =
        TransactionMessage::try_compile(&treasury_address, &instructions, &[]).unwrap();
    // Act
    let transaction_message_bytes = borsh::to_vec(&transaction_message).unwrap();
    let final_buffer_size = transaction_message_bytes.len() as u16;

    let final_buffer_hash = hashv(&[transaction_message_bytes.as_slice()]);
    governance_test
        .with_create_transaction_buffer(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            final_buffer_hash.to_bytes(),
            final_buffer_size,
            vec![], // Start with empty buffer
        )
        .await
        .unwrap();

    // Process the buffer in chunks
    governance_test
        .process_buffer_in_chunks(
            &mut proposal_cookie,
            &governance_cookie,
            transaction_message_bytes,
            700, // chunk size
            0,   // buffer index
        )
        .await
        .unwrap();

    let proposal_transaction_cookie = governance_test
        .with_insert_versioned_transaction_from_buffer(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            0,
            None,
            0,
        )
        .await
        .unwrap();
    governance_test
        .sign_off_proposal(&proposal_cookie, &signatory_record_cookie)
        .await
        .unwrap();

    governance_test
        .with_cast_yes_no_vote(&proposal_cookie, &token_owner_record_cookie, YesNoVote::Yes)
        .await
        .unwrap();

    // // Advance timestamp past hold_up_time
    governance_test
        .advance_clock_by_min_timespan(
            governance_cookie
                .account
                .config
                .min_transaction_hold_up_time as u64,
        )
        .await;

    let clock = governance_test.bench.get_clock().await;
    // Act
    governance_test
        .with_execute_versioned_transaction(
            &proposal_cookie,
            &proposal_transaction_cookie,
            transaction_message,
            0,
            0,
            &treasury_address,
            &proposal_cookie.account.governance,
            &[],
        )
        .await
        .unwrap();

    // Assert

    let proposal_account = governance_test
        .get_proposal_account(&proposal_cookie.address)
        .await;

    let yes_option = proposal_account.options.first().unwrap();

    assert_eq!(1, yes_option.transactions_executed_count);
    assert_eq!(ProposalState::Completed, proposal_account.state);
    assert_eq!(Some(clock.unix_timestamp), proposal_account.closed_at);
    assert_eq!(Some(clock.unix_timestamp), proposal_account.executing_at);

    let proposal_transaction_account = governance_test
        .get_proposal_versioned_transaction_account(&proposal_transaction_cookie.address)
        .await;

    assert_eq!(
        Some(clock.unix_timestamp),
        proposal_transaction_account.executed_at
    );

    assert_eq!(
        TransactionExecutionStatus::Success,
        proposal_transaction_account.execution_status
    );
}

#[tokio::test]
async fn test_create_mint_transaction_buffer_and_execute() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_cookie = governance_test.with_realm().await;
    let governed_account_cookie = governance_test.with_governed_account().await;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit(&realm_cookie)
        .await
        .unwrap();

    let mut governance_cookie = governance_test
        .with_governance(
            &realm_cookie,
            &governed_account_cookie,
            &token_owner_record_cookie,
        )
        .await
        .unwrap();

    governance_test
        .with_native_treasury(&governance_cookie)
        .await;

    let governed_mint_cookie = governance_test
        .with_governed_mint_governed_authority(&governance_cookie)
        .await;

    let mut proposal_cookie = governance_test
        .with_proposal(&token_owner_record_cookie, &mut governance_cookie)
        .await
        .unwrap();

    let signatory_record_cookie = governance_test
        .with_signatory(
            &proposal_cookie,
            &governance_cookie,
            &token_owner_record_cookie,
        )
        .await
        .unwrap();

    let treasury_address =
        get_native_treasury_address(&governance_test.program_id, &governance_cookie.address);

    let mut instructions = Vec::new();

    // Number of times to clone the instruction
    let instruction_count = 61;

    let token_account_keypair = Keypair::new();
    governance_test
        .bench
        .create_empty_token_account(
            &token_account_keypair,
            &governed_mint_cookie.address,
            &governance_test.bench.payer.pubkey(),
        )
        .await;

    let base_instruction = spl_token_2022::instruction::mint_to(
        &inline_spl_token::id(),
        &governed_mint_cookie.address,
        &token_account_keypair.pubkey(),
        &proposal_cookie.account.governance,
        &[],
        10,
    )
    .unwrap();
    // Fill the vec with cloned instructions
    for _ in 0..instruction_count {
        instructions.push(base_instruction.clone());
    }
    let transaction_message =
        TransactionMessage::try_compile(&proposal_cookie.account.governance, &instructions, &[])
            .unwrap();
    // Act
    let transaction_message_bytes = borsh::to_vec(&transaction_message).unwrap();
    let final_buffer_size = transaction_message_bytes.len() as u16;

    let final_buffer_hash = hashv(&[transaction_message_bytes.as_slice()]);
    governance_test
        .with_create_transaction_buffer(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            final_buffer_hash.to_bytes(),
            final_buffer_size,
            vec![], // Start with empty buffer
        )
        .await
        .unwrap();

    // Process the buffer in chunks
    governance_test
        .process_buffer_in_chunks(
            &mut proposal_cookie,
            &governance_cookie,
            transaction_message_bytes,
            700, // chunk size
            0,   // buffer index
        )
        .await
        .unwrap();

    let proposal_transaction_cookie = governance_test
        .with_insert_versioned_transaction_from_buffer(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            0,
            None,
            0,
        )
        .await
        .unwrap();
    governance_test
        .sign_off_proposal(&proposal_cookie, &signatory_record_cookie)
        .await
        .unwrap();

    governance_test
        .with_cast_yes_no_vote(&proposal_cookie, &token_owner_record_cookie, YesNoVote::Yes)
        .await
        .unwrap();

    // Advance timestamp past hold_up_time
    governance_test
        .advance_clock_by_min_timespan(
            governance_cookie
                .account
                .config
                .min_transaction_hold_up_time as u64,
        )
        .await;

    let clock = governance_test.bench.get_clock().await;

    // Act
    governance_test
        .with_execute_versioned_transaction(
            &proposal_cookie,
            &proposal_transaction_cookie,
            transaction_message,
            0,
            0,
            &treasury_address,
            &proposal_cookie.account.governance,
            &[],
        )
        .await
        .unwrap();

    // Assert

    let proposal_account = governance_test
        .get_proposal_account(&proposal_cookie.address)
        .await;

    let yes_option = proposal_account.options.first().unwrap();

    assert_eq!(1, yes_option.transactions_executed_count);
    assert_eq!(ProposalState::Completed, proposal_account.state);
    assert_eq!(Some(clock.unix_timestamp), proposal_account.closed_at);
    assert_eq!(Some(clock.unix_timestamp), proposal_account.executing_at);

    let proposal_transaction_account = governance_test
        .get_proposal_versioned_transaction_account(&proposal_transaction_cookie.address)
        .await;

    assert_eq!(
        Some(clock.unix_timestamp),
        proposal_transaction_account.executed_at
    );

    assert_eq!(
        TransactionExecutionStatus::Success,
        proposal_transaction_account.execution_status
    );
}

#[tokio::test]
async fn test_close_transaction_buffer() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_cookie = governance_test.with_realm().await;
    let governed_account_cookie = governance_test.with_governed_account().await;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit(&realm_cookie)
        .await
        .unwrap();

    let mut governance_cookie = governance_test
        .with_governance(
            &realm_cookie,
            &governed_account_cookie,
            &token_owner_record_cookie,
        )
        .await
        .unwrap();

    governance_test
        .with_native_treasury(&governance_cookie)
        .await;

    let mut proposal_cookie = governance_test
        .with_proposal(&token_owner_record_cookie, &mut governance_cookie)
        .await
        .unwrap();

    let treasury_address =
        get_native_treasury_address(&governance_test.program_id, &governance_cookie.address);

    let mut instructions = Vec::new();

    // Number of times to clone the instruction
    let instruction_count = 60;

    // Base instruction
    let base_instruction =
        system_instruction::transfer(&treasury_address, &Keypair::new().pubkey(), 1000000);

    // Fill the vec with cloned instructions
    for _ in 0..instruction_count {
        instructions.push(base_instruction.clone());
    }
    let transaction_message =
        TransactionMessage::try_compile(&proposal_cookie.account.governance, &instructions, &[])
            .unwrap();
    // Act
    let transaction_message_bytes = borsh::to_vec(&transaction_message).unwrap();
    let final_buffer_size = transaction_message_bytes.len() as u16;
    println!("{}", transaction_message_bytes.len());

    let final_buffer_hash = hashv(&[transaction_message_bytes.as_slice()]);
    let proposal_transaction_buffer_cookie = governance_test
        .with_create_transaction_buffer(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            final_buffer_hash.to_bytes(),
            final_buffer_size,
            vec![], // Start with empty buffer
        )
        .await
        .unwrap();

    // Process the buffer in chunks
    governance_test
        .process_buffer_in_chunks(
            &mut proposal_cookie,
            &governance_cookie,
            transaction_message_bytes,
            700, // chunk size
            0,   // buffer index
        )
        .await
        .unwrap();

    governance_test
        .with_close_transaction_buffer(
            &governance_cookie,
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
        )
        .await
        .unwrap();
    let proposal_transaction_buffer_account = governance_test
        .bench
        .get_account(&proposal_transaction_buffer_cookie.address)
        .await;

    assert_eq!(None, proposal_transaction_buffer_account);
}

#[tokio::test]
async fn test_transaction_buffer_exceeded_max_err() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_cookie = governance_test.with_realm().await;
    let governed_account_cookie = governance_test.with_governed_account().await;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit(&realm_cookie)
        .await
        .unwrap();

    let mut governance_cookie = governance_test
        .with_governance(
            &realm_cookie,
            &governed_account_cookie,
            &token_owner_record_cookie,
        )
        .await
        .unwrap();

    governance_test
        .with_native_treasury(&governance_cookie)
        .await;

    let governed_mint_cookie = governance_test
        .with_governed_mint_governed_authority(&governance_cookie)
        .await;

    let mut proposal_cookie = governance_test
        .with_proposal(&token_owner_record_cookie, &mut governance_cookie)
        .await
        .unwrap();

    let mut instructions = Vec::new();

    // Number of times to clone the instruction
    let instruction_count = 61;

    let token_account_keypair = Keypair::new();
    governance_test
        .bench
        .create_empty_token_account(
            &token_account_keypair,
            &governed_mint_cookie.address,
            &governance_test.bench.payer.pubkey(),
        )
        .await;

    let data: Vec<u8> = vec![1; 210];

    let base_instruction = Instruction {
        program_id: system_program::id(),
        accounts: vec![AccountMeta::new(proposal_cookie.account.governance, false)],
        data,
    };
    // Fill the vec with cloned instructions
    for _ in 0..instruction_count {
        instructions.push(base_instruction.clone());
    }
    let transaction_message =
        TransactionMessage::try_compile(&proposal_cookie.account.governance, &instructions, &[])
            .unwrap();
    // Act
    let transaction_message_bytes = borsh::to_vec(&transaction_message).unwrap();
    let final_buffer_size = transaction_message_bytes.len() as u16;

    let final_buffer_hash = hashv(&[transaction_message_bytes.as_slice()]);

    let err = governance_test
        .with_create_transaction_buffer(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            final_buffer_hash.to_bytes(),
            final_buffer_size,
            vec![], // Start with empty buffer
        )
        .await
        .err()
        .unwrap();

    assert_eq!(err, GovernanceError::FinalBufferSizeExceeded.into());
}

#[tokio::test]
async fn test_transaction_buffer_close_to_max() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_cookie = governance_test.with_realm().await;
    let governed_account_cookie = governance_test.with_governed_account().await;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit(&realm_cookie)
        .await
        .unwrap();

    let mut governance_cookie = governance_test
        .with_governance(
            &realm_cookie,
            &governed_account_cookie,
            &token_owner_record_cookie,
        )
        .await
        .unwrap();

    governance_test
        .with_native_treasury(&governance_cookie)
        .await;

    let governed_mint_cookie = governance_test
        .with_governed_mint_governed_authority(&governance_cookie)
        .await;

    let mut proposal_cookie = governance_test
        .with_proposal(&token_owner_record_cookie, &mut governance_cookie)
        .await
        .unwrap();

    let mut instructions = Vec::new();

    // Number of times to clone the instruction
    let instruction_count = 42;

    let token_account_keypair = Keypair::new();
    governance_test
        .bench
        .create_empty_token_account(
            &token_account_keypair,
            &governed_mint_cookie.address,
            &governance_test.bench.payer.pubkey(),
        )
        .await;

    let data: Vec<u8> = vec![1; 210];

    let base_instruction = Instruction {
        program_id: system_program::id(),
        accounts: vec![AccountMeta::new(proposal_cookie.account.governance, false)],
        data,
    };
    // Fill the vec with cloned instructions
    for _ in 0..instruction_count {
        instructions.push(base_instruction.clone());
    }
    let transaction_message =
        TransactionMessage::try_compile(&proposal_cookie.account.governance, &instructions, &[])
            .unwrap();
    // Act
    let transaction_message_bytes = borsh::to_vec(&transaction_message).unwrap();
    let final_buffer_size = transaction_message_bytes.len() as u16;

    let final_buffer_hash = hashv(&[transaction_message_bytes.as_slice()]);

    let _ = governance_test
        .with_create_transaction_buffer(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            final_buffer_hash.to_bytes(),
            final_buffer_size,
            vec![], // Start with empty buffer
        )
        .await;

    // Process the buffer in chunks
    governance_test
        .process_buffer_in_chunks(
            &mut proposal_cookie,
            &governance_cookie,
            transaction_message_bytes,
            700, // chunk size
            0,   // buffer index
        )
        .await
        .unwrap();

    let proposal_transaction_cookie = governance_test
        .with_insert_versioned_transaction_from_buffer(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            0,
            None,
            0,
        )
        .await
        .unwrap();

    // Assert

    let proposal_versioned_transaction_account = governance_test
        .get_proposal_versioned_transaction_account(&proposal_transaction_cookie.address)
        .await;

    assert_eq!(
        proposal_cookie.address,
        proposal_versioned_transaction_account.proposal
    );
}
