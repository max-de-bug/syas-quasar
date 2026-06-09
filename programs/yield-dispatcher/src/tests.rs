extern crate std;

use {
    quasar_svm::{Account, Instruction, Pubkey, QuasarSvm},
    solana_instruction::AccountMeta,
    spl_token_interface::state::{Account as TokenAccount, AccountState},
    std::{format, path::PathBuf, println, vec, vec::Vec},
    yield_adapter_trait::{
        KAMINO_ADAPTER_ID, MARGINFI_ADAPTER_ID, JUPITER_ADAPTER_ID,
        MAPLE_ADAPTER_ID, DRIFT_ADAPTER_ID, REGISTRY_PROGRAM_ID,
    },
};

const USER: Pubkey = Pubkey::new_from_array([1; 32]);
const MINT: Pubkey = Pubkey::new_from_array([2; 32]);
const USER_ATA: Pubkey = Pubkey::new_from_array([3; 32]);
const VAULT_ATA: Pubkey = Pubkey::new_from_array([4; 32]);

const DEPOSIT_AMOUNT: u64 = 1_000_000;
const WITHDRAW_SHARES: u64 = 500_000;

fn token_balance(account: &Account) -> u64 {
    u64::from_le_bytes(account.data[64..72].try_into().expect("amount bytes"))
}

fn deploy_elf(name: &str) -> Vec<u8> {
    let manifest = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        std::env::var("CARGO_TARGET_DIR")
            .ok()
            .map(|d| PathBuf::from(d).join(format!("deploy/{name}.so"))),
        Some(manifest.join(format!("target/deploy/{name}.so"))),
        Some(manifest.join(format!("../adapter-kamino/target/deploy/{name}.so"))),
        Some(manifest.join(format!("../adapter-registry/target/deploy/{name}.so"))),
        Some(manifest.join(format!("../adapter-marginfi/target/deploy/{name}.so"))),
        Some(manifest.join(format!("../adapter-jupiter/target/deploy/{name}.so"))),
        Some(manifest.join(format!("../adapter-maple/target/deploy/{name}.so"))),
        Some(manifest.join(format!("../adapter-drift/target/deploy/{name}.so"))),
    ];
    for path in candidates.into_iter().flatten() {
        if path.exists() {
            return std::fs::read(path).expect("read elf");
        }
    }
    panic!("build {name} first: cd programs/{name} && quasar build");
}

fn registry_id() -> Pubkey {
    Pubkey::from(REGISTRY_PROGRAM_ID.to_bytes())
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

fn init_registry_ix(authority: Pubkey, registry_state: Pubkey) -> Instruction {
    Instruction {
        program_id: registry_id(),
        accounts: vec![
            AccountMeta::new(authority, true),
            AccountMeta::new(registry_state, false),
            AccountMeta::new_readonly(quasar_svm::solana_sdk_ids::sysvar::rent::ID, false),
            AccountMeta::new_readonly(quasar_svm::system_program::ID, false),
        ],
        data: vec![0],
    }
}

fn propose_adapter_ix(
    proposer: Pubkey,
    registry_state: Pubkey,
    adapter_entry: Pubkey,
    adapter_program: Pubkey,
    underlying_mint: Pubkey,
    name: &str,
) -> Instruction {
    let mut data = vec![1];
    wincode::serialize_into(&mut data, &name).expect("name");
    wincode::serialize_into(&mut data, &"https://example.com/meta.json").expect("uri");
    Instruction {
        program_id: registry_id(),
        accounts: vec![
            AccountMeta::new(proposer, true),
            AccountMeta::new(registry_state, false),
            AccountMeta::new(adapter_entry, false),
            AccountMeta::new_readonly(adapter_program, false),
            AccountMeta::new_readonly(underlying_mint, false),
            AccountMeta::new_readonly(quasar_svm::solana_sdk_ids::sysvar::rent::ID, false),
            AccountMeta::new_readonly(quasar_svm::system_program::ID, false),
        ],
        data,
    }
}

fn approve_adapter_ix(authority: Pubkey, registry_state: Pubkey, adapter_entry: Pubkey) -> Instruction {
    Instruction {
        program_id: registry_id(),
        accounts: vec![
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(registry_state, false),
            AccountMeta::new(adapter_entry, false),
        ],
        data: vec![2],
    }
}

fn init_dispatcher_ix(authority: Pubkey, dispatcher_state: Pubkey) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(authority, true),
            AccountMeta::new(dispatcher_state, false),
            AccountMeta::new_readonly(registry_id(), false),
            AccountMeta::new_readonly(quasar_svm::solana_sdk_ids::sysvar::rent::ID, false),
            AccountMeta::new_readonly(quasar_svm::system_program::ID, false),
        ],
        data: vec![0],
    }
}

fn init_adapter_ix(authority: Pubkey, vault_state: Pubkey, mint: Pubkey, adapter: Pubkey) -> Instruction {
    let mint_addr = solana_address::Address::from(<[u8; 32]>::try_from(mint.as_ref()).unwrap());
    let mut data = vec![0u8];
    wincode::serialize_into(&mut data, &mint_addr).expect("serialize mint");
    Instruction {
        program_id: adapter,
        accounts: vec![
            AccountMeta::new(authority, true),
            AccountMeta::new(vault_state, false),
            AccountMeta::new_readonly(quasar_svm::solana_sdk_ids::sysvar::rent::ID, false),
            AccountMeta::new_readonly(quasar_svm::system_program::ID, false),
        ],
        data,
    }
}

fn dispatcher_deposit_ix(
    user: Pubkey,
    dispatcher_state: Pubkey,
    user_position: Pubkey,
    adapter_entry: Pubkey,
    adapter_prog: Pubkey,
    adapter_vault_state: Pubkey,
    adapter_vault: Pubkey,
    adapter_vault_authority: Pubkey,
    adapter_user_position: Pubkey,
    amount: u64,
) -> Instruction {
    let mut data = vec![1];
    data.extend_from_slice(&amount.to_le_bytes());
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new(dispatcher_state, false),
            AccountMeta::new(user_position, false),
            AccountMeta::new_readonly(registry_id(), false),
            AccountMeta::new_readonly(adapter_entry, false),
            AccountMeta::new_readonly(adapter_prog, false),
            AccountMeta::new(USER_ATA, false),
            AccountMeta::new(adapter_vault_state, false),
            AccountMeta::new(adapter_vault, false),
            AccountMeta::new_readonly(adapter_vault_authority, false),
            AccountMeta::new(adapter_user_position, false),
            AccountMeta::new_readonly(quasar_svm::SPL_TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(quasar_svm::system_program::ID, false),
        ],
        data,
    }
}

fn dispatcher_withdraw_ix(
    user: Pubkey,
    dispatcher_state: Pubkey,
    user_position: Pubkey,
    adapter_entry: Pubkey,
    adapter_prog: Pubkey,
    adapter_vault_state: Pubkey,
    adapter_vault: Pubkey,
    adapter_vault_authority: Pubkey,
    adapter_user_position: Pubkey,
    shares: u64,
) -> Instruction {
    let mut data = vec![2];
    data.extend_from_slice(&shares.to_le_bytes());
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new(dispatcher_state, false),
            AccountMeta::new(user_position, false),
            AccountMeta::new_readonly(registry_id(), false),
            AccountMeta::new_readonly(adapter_entry, false),
            AccountMeta::new_readonly(adapter_prog, false),
            AccountMeta::new(USER_ATA, false),
            AccountMeta::new(adapter_vault_state, false),
            AccountMeta::new(adapter_vault, false),
            AccountMeta::new_readonly(adapter_vault_authority, false),
            AccountMeta::new(adapter_user_position, false),
            AccountMeta::new_readonly(quasar_svm::SPL_TOKEN_PROGRAM_ID, false),
        ],
        data,
    }
}

fn dispatcher_current_value_ix(
    user: Pubkey,
    dispatcher_state: Pubkey,
    user_position: Pubkey,
    adapter_entry: Pubkey,
    adapter_prog: Pubkey,
    adapter_vault_state: Pubkey,
    adapter_user_position: Pubkey,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(user, true),
            AccountMeta::new_readonly(dispatcher_state, false),
            AccountMeta::new_readonly(user_position, false),
            AccountMeta::new_readonly(registry_id(), false),
            AccountMeta::new_readonly(adapter_entry, false),
            AccountMeta::new_readonly(adapter_prog, false),
            AccountMeta::new(adapter_vault_state, false),
            AccountMeta::new_readonly(adapter_user_position, false),
        ],
        data: vec![3],
    }
}

struct AdapterInfo {
    id: Pubkey,
    vault_state_seed: &'static [u8],
    vault_authority_seed: &'static [u8],
    elf_name: &'static str,
}

fn run_adapter_flow(adapter: &AdapterInfo) {
    let mut svm = QuasarSvm::new()
        .with_program(&crate::ID, &deploy_elf("yield_dispatcher"))
        .with_program(&registry_id(), &deploy_elf("adapter_registry"))
        .with_program(&adapter.id, &deploy_elf(adapter.elf_name))
        .with_token_program();

    let user = USER;
    let mint = MINT;
    let registry = registry_id();

    let (registry_state, _) = pda(&[b"registry_state"], &registry);
    let (adapter_entry, _) = pda(&[b"adapter_entry", adapter.id.as_ref()], &registry);
    let (dispatcher_state, _) = pda(&[b"dispatcher_state"], &crate::ID);
    let (vault_state, _) = pda(&[adapter.vault_state_seed], &adapter.id);
    let (vault_authority, _) = pda(&[adapter.vault_authority_seed], &adapter.id);
    let (user_position, _) = pda(&[b"adapter_position", user.as_ref()], &adapter.id);
    let (dispatcher_user_position, _) =
        pda(&[b"user_position", user.as_ref(), adapter.id.as_ref()], &crate::ID);

    let init_reg = svm.process_instruction(
        &init_registry_ix(user, registry_state),
        &[signer(user, 10_000_000_000), empty(registry_state)],
    );
    assert!(init_reg.is_ok(), "registry init: {:?}", init_reg.raw_result);

    let propose = svm.process_instruction(
        &propose_adapter_ix(user, registry_state, adapter_entry, adapter.id, mint, "Test Adapter"),
        &[
            signer(user, 10_000_000_000),
            init_reg.account(&registry_state).unwrap().clone(),
            empty(adapter_entry),
        ],
    );
    assert!(propose.is_ok(), "propose: {:?}", propose.raw_result);

    let approve = svm.process_instruction(
        &approve_adapter_ix(user, registry_state, adapter_entry),
        &[
            signer(user, 10_000_000_000),
            propose.account(&registry_state).unwrap().clone(),
            propose.account(&adapter_entry).unwrap().clone(),
        ],
    );
    assert!(approve.is_ok(), "approve: {:?}", approve.raw_result);

    let init_disp = svm.process_instruction(
        &init_dispatcher_ix(user, dispatcher_state),
        &[signer(user, 10_000_000_000), empty(dispatcher_state)],
    );
    assert!(init_disp.is_ok(), "dispatcher init: {:?}", init_disp.raw_result);

    let init_adap = svm.process_instruction(
        &init_adapter_ix(user, vault_state, mint, adapter.id),
        &[signer(user, 10_000_000_000), empty(vault_state)],
    );
    assert!(init_adap.is_ok(), "adapter init: {:?}", init_adap.raw_result);

    let deposit = svm.process_instruction(
        &dispatcher_deposit_ix(
            user, dispatcher_state, dispatcher_user_position,
            adapter_entry, adapter.id, vault_state, VAULT_ATA,
            vault_authority, user_position, DEPOSIT_AMOUNT,
        ),
        &[
            signer(user, 10_000_000_000),
            init_disp.account(&dispatcher_state).unwrap().clone(),
            empty(dispatcher_user_position),
            approve.account(&adapter_entry).unwrap().clone(),
            init_adap.account(&vault_state).unwrap().clone(),
            token_account(USER_ATA, mint, user, DEPOSIT_AMOUNT * 2),
            token_account(VAULT_ATA, mint, vault_authority, 0),
            empty(vault_authority),
            empty(user_position),
        ],
    );
    assert!(deposit.is_ok(), "dispatcher deposit: {:?}", deposit.raw_result);

    let vault_balance = token_balance(deposit.account(&VAULT_ATA).unwrap());
    assert_eq!(vault_balance, DEPOSIT_AMOUNT, "vault received underlying");

    let value = svm.process_instruction(
        &dispatcher_current_value_ix(
            user, dispatcher_state, dispatcher_user_position,
            adapter_entry, adapter.id, vault_state, user_position,
        ),
        &[
            signer(user, 10_000_000_000),
            deposit.account(&dispatcher_state).unwrap().clone(),
            deposit.account(&dispatcher_user_position).unwrap().clone(),
            approve.account(&adapter_entry).unwrap().clone(),
            deposit.account(&vault_state).unwrap().clone(),
            deposit.account(&user_position).unwrap().clone(),
        ],
    );
    assert!(value.is_ok(), "current_value: {:?}", value.raw_result);

    let withdraw = svm.process_instruction(
        &dispatcher_withdraw_ix(
            user, dispatcher_state, dispatcher_user_position,
            adapter_entry, adapter.id, vault_state, VAULT_ATA,
            vault_authority, user_position, WITHDRAW_SHARES,
        ),
        &[
            signer(user, 10_000_000_000),
            value.account(&dispatcher_state).unwrap().clone(),
            value.account(&dispatcher_user_position).unwrap().clone(),
            approve.account(&adapter_entry).unwrap().clone(),
            value.account(&USER_ATA).unwrap().clone(),
            value.account(&vault_state).unwrap().clone(),
            value.account(&VAULT_ATA).unwrap().clone(),
            empty(vault_authority),
            value.account(&user_position).unwrap().clone(),
        ],
    );
    assert!(withdraw.is_ok(), "dispatcher withdraw: {:?}", withdraw.raw_result);
    assert!(
        token_balance(withdraw.account(&USER_ATA).unwrap()) > DEPOSIT_AMOUNT,
        "user should receive withdrawn underlying"
    );

    println!(
        "  {} deposit CU: {}, value CU: {}, withdraw CU: {}",
        adapter.elf_name,
        deposit.compute_units_consumed,
        value.compute_units_consumed,
        withdraw.compute_units_consumed,
    );
}

#[test]
fn program_elf_loads() {
    let _svm = QuasarSvm::new()
        .with_program(&crate::ID, &deploy_elf("yield_dispatcher"));
}

#[test]
fn deposit_routes_through_kamino() {
    run_adapter_flow(&AdapterInfo {
        id: Pubkey::from(KAMINO_ADAPTER_ID.to_bytes()),
        vault_state_seed: b"kamino_vault_state",
        vault_authority_seed: b"kamino_vault_authority",
        elf_name: "adapter_kamino",
    });
}

#[test]
fn deposit_routes_through_marginfi() {
    run_adapter_flow(&AdapterInfo {
        id: Pubkey::from(MARGINFI_ADAPTER_ID.to_bytes()),
        vault_state_seed: b"marginfi_vault_state",
        vault_authority_seed: b"marginfi_vault_authority",
        elf_name: "adapter_marginfi",
    });
}

#[test]
fn deposit_routes_through_jupiter() {
    run_adapter_flow(&AdapterInfo {
        id: Pubkey::from(JUPITER_ADAPTER_ID.to_bytes()),
        vault_state_seed: b"jupiter_vault_state",
        vault_authority_seed: b"jupiter_vault_authority",
        elf_name: "adapter_jupiter",
    });
}

#[test]
fn deposit_routes_through_maple() {
    run_adapter_flow(&AdapterInfo {
        id: Pubkey::from(MAPLE_ADAPTER_ID.to_bytes()),
        vault_state_seed: b"maple_vault_state",
        vault_authority_seed: b"maple_vault_authority",
        elf_name: "adapter_maple",
    });
}

#[test]
fn deposit_routes_through_drift() {
    run_adapter_flow(&AdapterInfo {
        id: Pubkey::from(DRIFT_ADAPTER_ID.to_bytes()),
        vault_state_seed: b"drift_vault_state",
        vault_authority_seed: b"drift_vault_authority",
        elf_name: "adapter_drift",
    });
}
