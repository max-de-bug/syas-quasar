extern crate std;

use {
    quasar_svm::{Account, Instruction, Pubkey, QuasarSvm},
    solana_instruction::AccountMeta,
    spl_token_interface::state::{Account as TokenAccount, AccountState},
    std::{path::PathBuf, println, vec, vec::Vec},
};

use crate::protocol::MARGINFI_V2_ID;

const USER: Pubkey = Pubkey::new_from_array([1; 32]);
const MINT: Pubkey = Pubkey::new_from_array([2; 32]);
const USER_ATA: Pubkey = Pubkey::new_from_array([3; 32]);
const VAULT_ATA: Pubkey = Pubkey::new_from_array([4; 32]);

const DEPOSIT_AMOUNT: u64 = 1_000_000;
const WITHDRAW_SHARES: u64 = 500_000;

fn deploy_elf_path() -> PathBuf {
    if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
        let path = PathBuf::from(dir).join("deploy/adapter_marginfi.so");
        if path.exists() {
            return path;
        }
    }
    let manifest = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    if let Some(workspace) = manifest.parent().and_then(|p| p.parent()) {
        let path = workspace.join("target/deploy/adapter_marginfi.so");
        if path.exists() {
            return path;
        }
    }
    manifest.join("target/deploy/adapter_marginfi.so")
}

fn setup() -> QuasarSvm {
    let elf = std::fs::read(deploy_elf_path()).unwrap_or_else(|e| {
        panic!("build first: cd programs/adapter-marginfi && quasar build ({e})")
    });
    QuasarSvm::new()
        .with_program(&crate::ID, &elf)
        .with_token_program()
}

fn signer(pk: Pubkey, lamports: u64) -> Account {
    quasar_svm::token::create_keyed_system_account(&pk, lamports)
}

fn empty(pk: Pubkey) -> Account {
    Account {
        address: pk,
        lamports: 0,
        data: vec![],
        owner: quasar_svm::system_program::ID,
        executable: false,
    }
}

fn token_account(pk: Pubkey, mint: Pubkey, owner: Pubkey, amount: u64) -> Account {
    quasar_svm::token::create_keyed_token_account(
        &pk,
        &TokenAccount {
            mint,
            owner,
            amount,
            state: AccountState::Initialized,
            ..TokenAccount::default()
        },
    )
}

fn pda(seeds: &[&[u8]], program: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(seeds, program)
}

fn token_balance(account: &Account) -> u64 {
    u64::from_le_bytes(account.data[64..72].try_into().expect("amount bytes"))
}

fn init_ix(authority: Pubkey, vault_state: Pubkey, mint: Pubkey) -> Instruction {
    let mint_addr = solana_address::Address::from(<[u8; 32]>::try_from(mint.as_ref()).unwrap());
    let mut data = vec![0u8];
    wincode::serialize_into(&mut data, &mint_addr).expect("serialize mint");
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(authority, true),
            AccountMeta::new(vault_state, false),
            AccountMeta::new_readonly(quasar_svm::solana_sdk_ids::sysvar::rent::ID, false),
            AccountMeta::new_readonly(quasar_svm::system_program::ID, false),
        ],
        data,
    }
}

fn deposit_ix(
    user: Pubkey,
    vault_state: Pubkey,
    user_position: Pubkey,
    vault_authority: Pubkey,
    amount: u64,
) -> Instruction {
    let mut data = vec![1];
    data.extend_from_slice(&amount.to_le_bytes());
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new(vault_state, false),
            AccountMeta::new(user_position, false),
            AccountMeta::new(USER_ATA, false),
            AccountMeta::new_readonly(vault_authority, false),
            AccountMeta::new(VAULT_ATA, false),
            AccountMeta::new_readonly(quasar_svm::SPL_TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(quasar_svm::system_program::ID, false),
        ],
        data,
    }
}

fn current_value_ix(
    user: Pubkey,
    vault_state: Pubkey,
    user_position: Pubkey,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(user, true),
            AccountMeta::new(vault_state, false),
            AccountMeta::new_readonly(user_position, false),
        ],
        data: vec![3],
    }
}

fn withdraw_ix(
    user: Pubkey,
    vault_state: Pubkey,
    user_position: Pubkey,
    vault_authority: Pubkey,
    shares: u64,
) -> Instruction {
    let mut data = vec![2];
    data.extend_from_slice(&shares.to_le_bytes());
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new(vault_state, false),
            AccountMeta::new(user_position, false),
            AccountMeta::new(USER_ATA, false),
            AccountMeta::new(VAULT_ATA, false),
            AccountMeta::new_readonly(vault_authority, false),
            AccountMeta::new_readonly(quasar_svm::SPL_TOKEN_PROGRAM_ID, false),
        ],
        data,
    }
}

#[test]
fn program_elf_loads() {
    let _svm = setup();
}

#[test]
fn deposit_current_value_withdraw() {
    let mut svm = setup();

    let user = USER;
    let mint = MINT;

    let (vault_state, _) = pda(&[b"marginfi_vault_state"], &crate::ID);
    let (vault_authority, _) = pda(&[b"marginfi_vault_authority"], &crate::ID);
    let (user_position, _) = pda(&[b"adapter_position", user.as_ref()], &crate::ID);

    let init = svm.process_instruction(
        &init_ix(user, vault_state, mint),
        &[signer(user, 10_000_000_000), empty(vault_state)],
    );
    assert!(init.is_ok(), "initialize: {:?}", init.raw_result);

    let deposit = svm.process_instruction(
        &deposit_ix(user, vault_state, user_position, vault_authority, DEPOSIT_AMOUNT),
        &[
            signer(user, 10_000_000_000),
            init.account(&vault_state).unwrap().clone(),
            empty(user_position),
            token_account(USER_ATA, mint, user, DEPOSIT_AMOUNT * 2),
            empty(vault_authority),
            token_account(VAULT_ATA, mint, vault_authority, 0),
        ],
    );
    assert!(deposit.is_ok(), "deposit: {:?}", deposit.raw_result);
    assert_eq!(
        token_balance(deposit.account(&VAULT_ATA).unwrap()),
        DEPOSIT_AMOUNT
    );

    let value = svm.process_instruction(
        &current_value_ix(user, vault_state, user_position),
        &[
            signer(user, 10_000_000_000),
            deposit.account(&vault_state).unwrap().clone(),
            deposit.account(&user_position).unwrap().clone(),
        ],
    );
    assert!(value.is_ok(), "current_value: {:?}", value.raw_result);

    let withdraw = svm.process_instruction(
        &withdraw_ix(user, vault_state, user_position, vault_authority, WITHDRAW_SHARES),
        &[
            signer(user, 10_000_000_000),
            value.account(&vault_state).unwrap().clone(),
            value.account(&user_position).unwrap().clone(),
            value.account(&USER_ATA).unwrap().clone(),
            value.account(&VAULT_ATA).unwrap().clone(),
            empty(vault_authority),
        ],
    );
    assert!(withdraw.is_ok(), "withdraw: {:?}", withdraw.raw_result);
    assert!(token_balance(withdraw.account(&USER_ATA).unwrap()) > DEPOSIT_AMOUNT);

    println!(
        "  deposit CU: {}, value CU: {}, withdraw CU: {}",
        deposit.compute_units_consumed,
        value.compute_units_consumed,
        withdraw.compute_units_consumed
    );
}

fn deposit_with_remaining_ix(
    user: Pubkey,
    vault_state: Pubkey,
    user_position: Pubkey,
    vault_authority: Pubkey,
    amount: u64,
    remaining: Vec<AccountMeta>,
) -> Instruction {
    let mut ix = deposit_ix(user, vault_state, user_position, vault_authority, amount);
    ix.accounts.extend(remaining);
    ix
}

#[test]
fn deposit_fails_with_unregistered_protocol_cpi() {
    let mut svm = setup();

    let user = USER;
    let mint = MINT;

    let (vault_state, _) = pda(&[b"marginfi_vault_state"], &crate::ID);
    let (vault_authority, _) = pda(&[b"marginfi_vault_authority"], &crate::ID);
    let (user_position, _) = pda(&[b"adapter_position", user.as_ref()], &crate::ID);

    let init = svm.process_instruction(
        &init_ix(user, vault_state, mint),
        &[signer(user, 10_000_000_000), empty(vault_state)],
    );
    assert!(init.is_ok(), "initialize: {:?}", init.raw_result);

    let program_id: Pubkey =
        Pubkey::new_from_array(MARGINFI_V2_ID.as_ref().try_into().unwrap());
    let group = Pubkey::new_from_array([11; 32]);
    let marginfi_account = Pubkey::new_from_array([12; 32]);
    let bank = Pubkey::new_from_array([13; 32]);
    let bank_liquidity_vault = Pubkey::new_from_array([14; 32]);

    let remaining = vec![
        AccountMeta::new_readonly(program_id, false),
        AccountMeta::new_readonly(group, false),
        AccountMeta::new(marginfi_account, false),
        AccountMeta::new(bank, false),
        AccountMeta::new(bank_liquidity_vault, false),
    ];

    let deposit = svm.process_instruction(
        &deposit_with_remaining_ix(
            user,
            vault_state,
            user_position,
            vault_authority,
            DEPOSIT_AMOUNT,
            remaining,
        ),
        &[
            signer(user, 10_000_000_000),
            init.account(&vault_state).unwrap().clone(),
            empty(user_position),
            token_account(USER_ATA, mint, user, DEPOSIT_AMOUNT * 2),
            empty(vault_authority),
            token_account(VAULT_ATA, mint, vault_authority, 0),
        ],
    );
    assert!(
        deposit.is_err(),
        "expected CPI to fail for unregistered protocol program"
    );
}
