extern crate std;

use {
    adapter_kamino_client::*,
    quasar_svm::{Account, Instruction, Pubkey, QuasarSvm},
    solana_address::Address,
    spl_token_interface::state::{Account as TokenAccount, AccountState},
    std::{path::PathBuf, println, vec},
};

const USER: Pubkey = Pubkey::new_from_array([1; 32]);
const MINT: Pubkey = Pubkey::new_from_array([2; 32]);
const USER_ATA: Pubkey = Pubkey::new_from_array([3; 32]);
const VAULT_ATA: Pubkey = Pubkey::new_from_array([4; 32]);

const DEPOSIT_AMOUNT: u64 = 1_000_000;
const WITHDRAW_SHARES: u64 = 500_000;

fn deploy_elf_path() -> PathBuf {
    if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
        let path = PathBuf::from(dir).join("deploy/adapter_kamino.so");
        if path.exists() {
            return path;
        }
    }
    PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("target/deploy/adapter_kamino.so")
}

fn setup() -> QuasarSvm {
    let elf = std::fs::read(deploy_elf_path())
        .unwrap_or_else(|e| panic!("build first: cd programs/adapter-kamino && quasar build ({e})"));
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

fn to_address(pk: Pubkey) -> Address {
    Address::from(<[u8; 32]>::try_from(pk.as_ref()).expect("pubkey length"))
}

fn to_pubkey(address: Address) -> Pubkey {
    Pubkey::new_from_array(*address.as_array())
}

fn token_balance(account: &Account) -> u64 {
    let data = &account.data;
    assert!(data.len() >= 72, "token account data too short");
    u64::from_le_bytes(data[64..72].try_into().expect("amount bytes"))
}

#[test]
fn program_elf_loads() {
    let _svm = setup();
}

#[test]
fn deposit_current_value_withdraw() {
    let mut svm = setup();

    let program_id = to_address(crate::ID);
    let user = USER;
    let mint = MINT;

    let (vault_state, _) = find_vault_state_address(&program_id);
    let (vault_authority, _) = find_vault_authority_address(&program_id);
    let (user_position, _) = find_user_position_address(&to_address(user), &program_id);

    let vault_state_pk = to_pubkey(vault_state);
    let vault_authority_pk = to_pubkey(vault_authority);
    let user_position_pk = to_pubkey(user_position);

    let token_program = quasar_svm::SPL_TOKEN_PROGRAM_ID;
    let system_program = quasar_svm::system_program::ID;
    let rent = quasar_svm::solana_sdk_ids::sysvar::rent::ID;

    let init_ix: Instruction = InitializeInstruction {
        authority: to_address(user),
        vault_state,
        rent: to_address(rent),
        system_program: to_address(system_program),
        underlying_mint: to_address(mint),
    }
    .into();

    let init_result = svm.process_instruction(
        &init_ix,
        &[signer(user, 10_000_000_000), empty(vault_state_pk)],
    );
    assert!(
        init_result.is_ok(),
        "initialize failed: {:?}",
        init_result.raw_result
    );

    let deposit_ix: Instruction = DepositInstruction {
        user: to_address(user),
        vault_state,
        user_position,
        user_token_account: to_address(USER_ATA),
        vault_authority,
        vault_token_account: to_address(VAULT_ATA),
        token_program: to_address(token_program),
        system_program: to_address(system_program),
        amount: DEPOSIT_AMOUNT,
        remaining_accounts: vec![],
    }
    .into();

    let deposit_result = svm.process_instruction(
        &deposit_ix,
        &[
            signer(user, 10_000_000_000),
            init_result.account(&vault_state_pk).unwrap().clone(),
            empty(user_position_pk),
            token_account(USER_ATA, mint, user, DEPOSIT_AMOUNT * 2),
            empty(vault_authority_pk),
            token_account(VAULT_ATA, mint, vault_authority_pk, 0),
        ],
    );
    assert!(
        deposit_result.is_ok(),
        "deposit failed: {:?}",
        deposit_result.raw_result
    );
    assert_eq!(
        token_balance(deposit_result.account(&VAULT_ATA).unwrap()),
        DEPOSIT_AMOUNT,
        "vault balance after deposit"
    );

    let value_ix: Instruction = Current_valueInstruction {
        user: to_address(user),
        vault_state,
        user_position,
        remaining_accounts: vec![],
    }
    .into();

    let value_result = svm.process_instruction(
        &value_ix,
        &[
            signer(user, 10_000_000_000),
            deposit_result.account(&vault_state_pk).unwrap().clone(),
            deposit_result
                .account(&user_position_pk)
                .unwrap()
                .clone(),
        ],
    );
    assert!(
        value_result.is_ok(),
        "current_value failed: {:?}",
        value_result.raw_result
    );

    let withdraw_ix: Instruction = WithdrawInstruction {
        user: to_address(user),
        vault_state,
        user_position,
        user_token_account: to_address(USER_ATA),
        vault_token_account: to_address(VAULT_ATA),
        vault_authority,
        token_program: to_address(token_program),
        amount: WITHDRAW_SHARES,
    }
    .into();

    let withdraw_result = svm.process_instruction(
        &withdraw_ix,
        &[
            signer(user, 10_000_000_000),
            value_result.account(&vault_state_pk).unwrap().clone(),
            value_result
                .account(&user_position_pk)
                .unwrap()
                .clone(),
            value_result.account(&USER_ATA).unwrap().clone(),
            value_result.account(&VAULT_ATA).unwrap().clone(),
            empty(vault_authority_pk),
        ],
    );
    assert!(
        withdraw_result.is_ok(),
        "withdraw failed: {:?}",
        withdraw_result.raw_result
    );

    let user_balance = token_balance(withdraw_result.account(&USER_ATA).unwrap());
    assert!(
        user_balance > DEPOSIT_AMOUNT,
        "user should receive withdrawn underlying"
    );
    assert!(
        token_balance(withdraw_result.account(&VAULT_ATA).unwrap()) < DEPOSIT_AMOUNT,
        "vault should hold less after withdraw"
    );

    println!(
        "  deposit CU: {}, value CU: {}, withdraw CU: {}",
        deposit_result.compute_units_consumed,
        value_result.compute_units_consumed,
        withdraw_result.compute_units_consumed
    );
}
