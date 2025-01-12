#![cfg(feature = "test-sbf")]

mod program_test;

use {
    program_test::*,
    solana_program_test::tokio,
    solana_sdk::{signer::Signer, system_instruction},
    spl_governance::{
        error::GovernanceError, state::native_treasury::get_native_treasury_address,
        tools::transaction_message::TransactionMessage,
    },
    versioned_transaction_ext::VaultTransactionMessageExt,
};

#[tokio::test]
async fn test_remove_versioned_transaction() {
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

    let mut proposal_cookie = governance_test
        .with_proposal(&token_owner_record_cookie, &mut governance_cookie)
        .await
        .unwrap();

    let treasury_address =
        get_native_treasury_address(&governance_test.program_id, &governance_cookie.address);
    let instruction = system_instruction::transfer(
        &treasury_address,
        &governance_test.bench.payer.pubkey(),
        1_000_000_000,
    );
    let transaction_message =
        TransactionMessage::try_compile(&treasury_address, &[instruction], &[]).unwrap();
    // Act
    let transaction_message_bytes = borsh::to_vec(&transaction_message).unwrap();

    let proposal_vtransaction_cookie = governance_test
        .with_insert_versioned_transaction(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            0,
            None,
            transaction_message_bytes,
        )
        .await
        .unwrap();

    // Act

    governance_test
        .remove_versioned_transaction(
            &proposal_cookie,
            &token_owner_record_cookie,
            &proposal_vtransaction_cookie,
        )
        .await
        .unwrap();

    // Assert

    let proposal_account = governance_test
        .get_proposal_account(&proposal_cookie.address)
        .await;

    let yes_option = proposal_account.options.first().unwrap();

    assert_eq!(yes_option.transactions_count, 0);
    assert_eq!(yes_option.transactions_next_index, 1);
    assert_eq!(yes_option.transactions_executed_count, 0);

    let proposal_transaction_account = governance_test
        .bench
        .get_account(&proposal_vtransaction_cookie.address)
        .await;

    assert_eq!(None, proposal_transaction_account);
}

#[tokio::test]
async fn test_replace_versioned_transaction() {
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

    let mut proposal_cookie = governance_test
        .with_proposal(&token_owner_record_cookie, &mut governance_cookie)
        .await
        .unwrap();

    let treasury_address =
        get_native_treasury_address(&governance_test.program_id, &governance_cookie.address);
    let instruction = system_instruction::transfer(
        &treasury_address,
        &governance_test.bench.payer.pubkey(),
        1_000_000_000,
    );
    let transaction_message =
        TransactionMessage::try_compile(&treasury_address, &[instruction], &[]).unwrap();
    // Act
    let transaction_message_bytes = borsh::to_vec(&transaction_message).unwrap();

    let proposal_vtransaction_cookie = governance_test
        .with_insert_versioned_transaction(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            0,
            None,
            transaction_message_bytes.clone(),
        )
        .await
        .unwrap();
    governance_test
        .with_insert_versioned_transaction(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            0,
            None,
            transaction_message_bytes.clone(),
        )
        .await
        .unwrap();
    // Act

    governance_test
        .remove_versioned_transaction(
            &proposal_cookie,
            &token_owner_record_cookie,
            &proposal_vtransaction_cookie,
        )
        .await
        .unwrap();

    let proposal_transaction_cookie2 = governance_test
        .with_insert_versioned_transaction(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            0,
            Some(0),
            transaction_message_bytes.clone(),
        )
        .await
        .unwrap();

    // Assert
    let proposal_account = governance_test
        .get_proposal_account(&proposal_cookie.address)
        .await;

    let yes_option = proposal_account.options.first().unwrap();

    assert_eq!(yes_option.transactions_count, 2);
    assert_eq!(yes_option.transactions_next_index, 2);

    let proposal_transaction_account2 = governance_test
        .get_proposal_versioned_transaction_account(&proposal_transaction_cookie2.address)
        .await;

    assert_eq!(
        proposal_transaction_cookie2.option_index,
        proposal_transaction_account2.option_index
    );
}

#[tokio::test]
async fn test_remove_front_versioned_transaction() {
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

    let mut proposal_cookie = governance_test
        .with_proposal(&token_owner_record_cookie, &mut governance_cookie)
        .await
        .unwrap();

    let treasury_address =
        get_native_treasury_address(&governance_test.program_id, &governance_cookie.address);
    let instruction = system_instruction::transfer(
        &treasury_address,
        &governance_test.bench.payer.pubkey(),
        1_000_000_000,
    );
    let transaction_message =
        TransactionMessage::try_compile(&treasury_address, &[instruction], &[]).unwrap();
    // Act
    let transaction_message_bytes = borsh::to_vec(&transaction_message).unwrap();

    let proposal_vtransaction_cookie = governance_test
        .with_insert_versioned_transaction(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            0,
            None,
            transaction_message_bytes.clone(),
        )
        .await
        .unwrap();
    governance_test
        .with_insert_versioned_transaction(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            0,
            None,
            transaction_message_bytes.clone(),
        )
        .await
        .unwrap();
    // Act

    governance_test
        .remove_versioned_transaction(
            &proposal_cookie,
            &token_owner_record_cookie,
            &proposal_vtransaction_cookie,
        )
        .await
        .unwrap();
    // Assert
    let proposal_account = governance_test
        .get_proposal_account(&proposal_cookie.address)
        .await;

    let yes_option = proposal_account.options.first().unwrap();

    assert_eq!(yes_option.transactions_count, 1);
    assert_eq!(yes_option.transactions_next_index, 2);

    let proposal_transaction_account = governance_test
        .bench
        .get_account(&proposal_vtransaction_cookie.address)
        .await;

    assert_eq!(None, proposal_transaction_account);
}

#[tokio::test]
async fn test_remove_versioned_transaction_with_owner_or_delegate_must_sign_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_cookie = governance_test.with_realm().await;
    let governed_account_cookie = governance_test.with_governed_account().await;

    let mut token_owner_record_cookie = governance_test
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

    let mut proposal_cookie = governance_test
        .with_proposal(&token_owner_record_cookie, &mut governance_cookie)
        .await
        .unwrap();

    let treasury_address =
        get_native_treasury_address(&governance_test.program_id, &governance_cookie.address);
    let instruction = system_instruction::transfer(
        &treasury_address,
        &governance_test.bench.payer.pubkey(),
        1_000_000_000,
    );
    let transaction_message =
        TransactionMessage::try_compile(&treasury_address, &[instruction], &[]).unwrap();
    // Act
    let transaction_message_bytes = borsh::to_vec(&transaction_message).unwrap();

    let proposal_vtransaction_cookie = governance_test
        .with_insert_versioned_transaction(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            0,
            None,
            transaction_message_bytes.clone(),
        )
        .await
        .unwrap();
    governance_test
        .with_insert_versioned_transaction(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            0,
            None,
            transaction_message_bytes.clone(),
        )
        .await
        .unwrap();

    let token_owner_record_cookie2 = governance_test
        .with_council_token_deposit(&realm_cookie)
        .await
        .unwrap();

    token_owner_record_cookie.token_owner = token_owner_record_cookie2.token_owner;

    // Act
    let err = governance_test
        .remove_versioned_transaction(
            &proposal_cookie,
            &token_owner_record_cookie,
            &proposal_vtransaction_cookie,
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(
        err,
        GovernanceError::GoverningTokenOwnerOrDelegateMustSign.into()
    );
}

#[tokio::test]
async fn test_remove_versioned_transaction_with_proposal_not_editable_error() {
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

    let mut proposal_cookie = governance_test
        .with_proposal(&token_owner_record_cookie, &mut governance_cookie)
        .await
        .unwrap();

    let treasury_address =
        get_native_treasury_address(&governance_test.program_id, &governance_cookie.address);
    let instruction = system_instruction::transfer(
        &treasury_address,
        &governance_test.bench.payer.pubkey(),
        1_000_000_000,
    );
    let transaction_message =
        TransactionMessage::try_compile(&treasury_address, &[instruction], &[]).unwrap();
    // Act
    let transaction_message_bytes = borsh::to_vec(&transaction_message).unwrap();

    let proposal_vtransaction_cookie = governance_test
        .with_insert_versioned_transaction(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            0,
            None,
            transaction_message_bytes.clone(),
        )
        .await
        .unwrap();

    governance_test
        .cancel_proposal(&proposal_cookie, &token_owner_record_cookie)
        .await
        .unwrap();

    // Act
    let err = governance_test
        .remove_versioned_transaction(
            &proposal_cookie,
            &token_owner_record_cookie,
            &proposal_vtransaction_cookie,
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(
        err,
        GovernanceError::InvalidStateCannotEditTransactions.into()
    );
}

#[tokio::test]
async fn test_remove_versioned_transaction_with_proposal_versioned_transaction_from_other_proposal_error(
) {
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

    let mut proposal_cookie = governance_test
        .with_proposal(&token_owner_record_cookie, &mut governance_cookie)
        .await
        .unwrap();

    let treasury_address =
        get_native_treasury_address(&governance_test.program_id, &governance_cookie.address);
    let instruction = system_instruction::transfer(
        &treasury_address,
        &governance_test.bench.payer.pubkey(),
        1_000_000_000,
    );
    let transaction_message =
        TransactionMessage::try_compile(&treasury_address, &[instruction.clone()], &[]).unwrap();
    // Act
    let transaction_message_bytes = borsh::to_vec(&transaction_message).unwrap();

    governance_test
        .with_insert_versioned_transaction(
            &mut proposal_cookie,
            &token_owner_record_cookie,
            0,
            0,
            None,
            transaction_message_bytes.clone(),
        )
        .await
        .unwrap();

    let token_owner_record_cookie2 = governance_test
        .with_community_token_deposit(&realm_cookie)
        .await
        .unwrap();

    let mut proposal_cookie2 = governance_test
        .with_proposal(&token_owner_record_cookie2, &mut governance_cookie)
        .await
        .unwrap();

    let transaction_message =
        TransactionMessage::try_compile(&treasury_address, &[instruction.clone()], &[]).unwrap();
    // Act
    let transaction_message_bytes = borsh::to_vec(&transaction_message).unwrap();

    let proposal_vtransaction_cookie_2 = governance_test
        .with_insert_versioned_transaction(
            &mut proposal_cookie2,
            &token_owner_record_cookie2,
            0,
            0,
            None,
            transaction_message_bytes.clone(),
        )
        .await
        .unwrap();
    // Act
    let err = governance_test
        .remove_versioned_transaction(
            &proposal_cookie,
            &token_owner_record_cookie,
            &proposal_vtransaction_cookie_2,
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(
        err,
        GovernanceError::InvalidProposalForProposalTransaction.into()
    );
}
