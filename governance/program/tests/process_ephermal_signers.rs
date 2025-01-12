#![cfg(feature = "test-sbf")]

mod program_test;

use {
    program_test::*,
    solana_program_test::tokio,
    solana_sdk::{signature::Keypair, signer::Signer},
    spl_governance::{
        state::{
            enums::{ProposalState, TransactionExecutionStatus},
            native_treasury::get_native_treasury_address,
            proposal_versioned_transaction::get_proposal_versioned_transaction_address,
        },
        tools::transaction_message::TransactionMessage,
    },
    spl_governance_test_sdk::{
        mpl_core_tools::{
            assert_asset, assert_collection, create_asset, create_collection,
            AssertAssetHelperArgs, AssertCollectionHelperArgs, CreateAssetHelperArgs,
            CreateCollectionHelperArgs, UpdateAuthority,
        },
        versioned_transaction::get_ephemeral_signer_pda,
    },
    versioned_transaction_ext::VaultTransactionMessageExt,
};

#[tokio::test]
async fn test_create_asset_mpl_core_via_versioned_transaction() {
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

    let governed_mint_cookie = governance_test.with_governed_mint().await;

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

    let token_account_keypair = Keypair::new();
    governance_test
        .bench
        .create_empty_token_account(
            &token_account_keypair,
            &governed_mint_cookie.address,
            &governance_test.bench.payer.pubkey(),
        )
        .await;
    let proposal_versioned_tx_address = get_proposal_versioned_transaction_address(
        &governance_test.program_id,
        &proposal_cookie.address,
        &0_u8.to_le_bytes(),
        &0_u16.to_le_bytes(),
    );
    let (asset_pubkey, _bump) = get_ephemeral_signer_pda(
        &proposal_versioned_tx_address,
        0,
        &governance_test.program_id,
        0,
    );
    let instruction = create_asset(
        CreateAssetHelperArgs {
            owner: None,
            payer: None,
            asset: &asset_pubkey,
            data_state: None,
            name: None,
            uri: None,
            authority: None,
            update_authority: None,
            collection: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
        treasury_address,
    );

    let transaction_message =
        TransactionMessage::try_compile(&proposal_cookie.account.governance, &[instruction], &[])
            .unwrap();
    // Act
    let transaction_message_bytes = borsh::to_vec(&transaction_message).unwrap();

    let proposal_transaction_cookie = governance_test
        .with_insert_versioned_transaction(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            1,
            None,
            transaction_message_bytes,
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
            1,
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

    assert_asset(
        &mut governance_test.bench.context,
        AssertAssetHelperArgs {
            asset: asset_pubkey,
            owner: treasury_address,
            update_authority: Some(UpdateAuthority::Address(treasury_address)),
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await;
}

#[tokio::test]
async fn test_create_collection_mpl_core_via_versioned_transaction() {
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

    let governed_mint_cookie = governance_test.with_governed_mint().await;

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

    let token_account_keypair = Keypair::new();
    governance_test
        .bench
        .create_empty_token_account(
            &token_account_keypair,
            &governed_mint_cookie.address,
            &governance_test.bench.payer.pubkey(),
        )
        .await;
    let proposal_versioned_tx_address = get_proposal_versioned_transaction_address(
        &governance_test.program_id,
        &proposal_cookie.address,
        &0_u8.to_le_bytes(),
        &0_u16.to_le_bytes(),
    );
    let (collection_pubkey, _bump) = get_ephemeral_signer_pda(
        &proposal_versioned_tx_address,
        0,
        &governance_test.program_id,
        0,
    );
    let instruction = create_collection(
        CreateCollectionHelperArgs {
            collection: &collection_pubkey,
            update_authority: None,
            payer: None,
            name: None,
            uri: None,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
        treasury_address,
    );

    let transaction_message =
        TransactionMessage::try_compile(&proposal_cookie.account.governance, &[instruction], &[])
            .unwrap();
    // Act
    let transaction_message_bytes = borsh::to_vec(&transaction_message).unwrap();

    let proposal_transaction_cookie = governance_test
        .with_insert_versioned_transaction(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            1,
            None,
            transaction_message_bytes,
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
            1,
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

    assert_collection(
        &mut governance_test.bench.context,
        AssertCollectionHelperArgs {
            collection: collection_pubkey,
            update_authority: treasury_address,
            name: None,
            uri: None,
            num_minted: 0,
            current_size: 0,
            plugins: vec![],
            external_plugin_adapters: vec![],
        },
    )
    .await;
}
