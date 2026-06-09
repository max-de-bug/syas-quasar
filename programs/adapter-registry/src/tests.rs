extern crate std;

use {
    quasar_svm::{Account, Instruction, Pubkey, QuasarSvm},
    solana_instruction::AccountMeta,
    std::{path::PathBuf, println, vec},
};

const AUTHORITY: Pubkey = Pubkey::new_from_array([1; 32]);
const ADAPTER_PROGRAM: Pubkey = Pubkey::new_from_array([2; 32]);
const UNDERLYING_MINT: Pubkey = Pubkey::new_from_array([3; 32]);
const NEW_AUTHORITY: Pubkey = Pubkey::new_from_array([4; 32]);

const ADAPTER_NAME: &str = "Test Adapter";
const METADATA_URI: &str = "https://example.com/metadata.json";

fn deploy_elf_path() -> PathBuf {
    if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
        let path = PathBuf::from(dir).join("deploy/adapter_registry.so");
        if path.exists() {
            return path;
        }
    }
    PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("target/deploy/adapter_registry.so")
}

fn setup() -> QuasarSvm {
    let elf = std::fs::read(deploy_elf_path())
        .unwrap_or_else(|e| panic!("build first: cd programs/adapter-registry && quasar build ({e})"));
    QuasarSvm::new().with_program(&crate::ID, &elf)
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

fn find_registry_state(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"registry_state"], program_id)
}

fn find_adapter_entry(adapter_program: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"adapter_entry", adapter_program.as_ref()],
        program_id,
    )
}

fn initialize_ix(authority: Pubkey, registry_state: Pubkey) -> Instruction {
    Instruction {
        program_id: crate::ID,
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
    metadata_uri: &str,
) -> Instruction {
    let mut data = vec![1];
    wincode::serialize_into(&mut data, &name).expect("serialize name");
    wincode::serialize_into(&mut data, &metadata_uri).expect("serialize metadata_uri");
    Instruction {
        program_id: crate::ID,
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

fn approve_adapter_ix(
    authority: Pubkey,
    registry_state: Pubkey,
    adapter_entry: Pubkey,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(registry_state, false),
            AccountMeta::new(adapter_entry, false),
        ],
        data: vec![2],
    }
}

fn revoke_adapter_ix(
    authority: Pubkey,
    registry_state: Pubkey,
    adapter_entry: Pubkey,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(registry_state, false),
            AccountMeta::new(adapter_entry, false),
        ],
        data: vec![3],
    }
}

fn transfer_governance_ix(
    authority: Pubkey,
    registry_state: Pubkey,
    new_authority: Pubkey,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(registry_state, false),
            AccountMeta::new_readonly(new_authority, false),
        ],
        data: vec![4],
    }
}

#[test]
fn program_elf_loads() {
    let _svm = setup();
}

#[test]
fn registry_governance_lifecycle() {
    let mut svm = setup();

    let program_id = crate::ID;
    let authority = AUTHORITY;
    let (registry_state, _) = find_registry_state(&program_id);
    let (adapter_entry, _) = find_adapter_entry(&ADAPTER_PROGRAM, &program_id);

    let init = svm.process_instruction(
        &initialize_ix(authority, registry_state),
        &[signer(authority, 10_000_000_000), empty(registry_state)],
    );
    assert!(init.is_ok(), "initialize failed: {:?}", init.raw_result);

    let propose = svm.process_instruction(
        &propose_adapter_ix(
            authority,
            registry_state,
            adapter_entry,
            ADAPTER_PROGRAM,
            UNDERLYING_MINT,
            ADAPTER_NAME,
            METADATA_URI,
        ),
        &[
            signer(authority, 10_000_000_000),
            init.account(&registry_state).unwrap().clone(),
            empty(adapter_entry),
        ],
    );
    assert!(propose.is_ok(), "propose failed: {:?}", propose.raw_result);

    let approve = svm.process_instruction(
        &approve_adapter_ix(authority, registry_state, adapter_entry),
        &[
            signer(authority, 10_000_000_000),
            propose.account(&registry_state).unwrap().clone(),
            propose.account(&adapter_entry).unwrap().clone(),
        ],
    );
    assert!(approve.is_ok(), "approve failed: {:?}", approve.raw_result);

    let revoke = svm.process_instruction(
        &revoke_adapter_ix(authority, registry_state, adapter_entry),
        &[
            signer(authority, 10_000_000_000),
            approve.account(&registry_state).unwrap().clone(),
            approve.account(&adapter_entry).unwrap().clone(),
        ],
    );
    assert!(revoke.is_ok(), "revoke failed: {:?}", revoke.raw_result);

    let transfer = svm.process_instruction(
        &transfer_governance_ix(authority, registry_state, NEW_AUTHORITY),
        &[
            signer(authority, 10_000_000_000),
            revoke.account(&registry_state).unwrap().clone(),
        ],
    );
    assert!(
        transfer.is_ok(),
        "transfer_governance failed: {:?}",
        transfer.raw_result
    );

    let state_data = &transfer.account(&registry_state).unwrap().data;
    assert_eq!(state_data[0], 1, "registry discriminator");
    assert_eq!(
        &state_data[1..33],
        NEW_AUTHORITY.as_ref(),
        "authority updated"
    );

    println!(
        "  init CU: {}, propose CU: {}, approve CU: {}, revoke CU: {}, transfer CU: {}",
        init.compute_units_consumed,
        propose.compute_units_consumed,
        approve.compute_units_consumed,
        revoke.compute_units_consumed,
        transfer.compute_units_consumed
    );
}

#[test]
fn unauthorized_approve_fails() {
    let mut svm = setup();

    let program_id = crate::ID;
    let authority = AUTHORITY;
    let intruder = Pubkey::new_from_array([9; 32]);
    let (registry_state, _) = find_registry_state(&program_id);
    let (adapter_entry, _) = find_adapter_entry(&ADAPTER_PROGRAM, &program_id);

    let init = svm.process_instruction(
        &initialize_ix(authority, registry_state),
        &[signer(authority, 10_000_000_000), empty(registry_state)],
    );
    assert!(init.is_ok(), "initialize: {:?}", init.raw_result);

    let propose = svm.process_instruction(
        &propose_adapter_ix(
            authority,
            registry_state,
            adapter_entry,
            ADAPTER_PROGRAM,
            UNDERLYING_MINT,
            ADAPTER_NAME,
            METADATA_URI,
        ),
        &[
            signer(authority, 10_000_000_000),
            init.account(&registry_state).unwrap().clone(),
            empty(adapter_entry),
        ],
    );
    assert!(propose.is_ok(), "propose: {:?}", propose.raw_result);

    let approve = svm.process_instruction(
        &approve_adapter_ix(intruder, registry_state, adapter_entry),
        &[
            signer(intruder, 10_000_000_000),
            propose.account(&registry_state).unwrap().clone(),
            propose.account(&adapter_entry).unwrap().clone(),
        ],
    );
    assert!(approve.is_err(), "intruder approve should fail");
}
