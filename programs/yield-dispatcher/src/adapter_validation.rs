//! Registry adapter entry and reference-adapter PDA validation without linking program crates.

use {
    crate::error::DispatcherError,
    quasar_lang::{pda::based_try_find_program_address, prelude::*},
    yield_adapter_trait::{
        adapter_entry_is_approved, read_adapter_entry_underlying_mint, ADAPTER_POSITION_SEED,
        KAMINO_ADAPTER_ID, KAMINO_VAULT_AUTHORITY_SEED, KAMINO_VAULT_STATE_SEED,
        JUPITER_ADAPTER_ID, JUPITER_VAULT_AUTHORITY_SEED, JUPITER_VAULT_STATE_SEED,
        DRIFT_ADAPTER_ID, DRIFT_VAULT_AUTHORITY_SEED, DRIFT_VAULT_STATE_SEED,
        MAPLE_ADAPTER_ID, MAPLE_VAULT_AUTHORITY_SEED, MAPLE_VAULT_STATE_SEED,
        MARGINFI_ADAPTER_ID, MARGINFI_VAULT_AUTHORITY_SEED, MARGINFI_VAULT_STATE_SEED,
        REGISTRY_ADAPTER_ENTRY_SEED, REGISTRY_PROGRAM_ID,
    },
};

struct AdapterVaultSeeds {
    vault_state: &'static [u8],
    vault_authority: &'static [u8],
}

fn vault_seeds(adapter: &Address) -> Option<AdapterVaultSeeds> {
    if *adapter == KAMINO_ADAPTER_ID {
        Some(AdapterVaultSeeds {
            vault_state: KAMINO_VAULT_STATE_SEED,
            vault_authority: KAMINO_VAULT_AUTHORITY_SEED,
        })
    } else if *adapter == MARGINFI_ADAPTER_ID {
        Some(AdapterVaultSeeds {
            vault_state: MARGINFI_VAULT_STATE_SEED,
            vault_authority: MARGINFI_VAULT_AUTHORITY_SEED,
        })
    } else if *adapter == JUPITER_ADAPTER_ID {
        Some(AdapterVaultSeeds {
            vault_state: JUPITER_VAULT_STATE_SEED,
            vault_authority: JUPITER_VAULT_AUTHORITY_SEED,
        })
    } else if *adapter == MAPLE_ADAPTER_ID {
        Some(AdapterVaultSeeds {
            vault_state: MAPLE_VAULT_STATE_SEED,
            vault_authority: MAPLE_VAULT_AUTHORITY_SEED,
        })
    } else if *adapter == DRIFT_ADAPTER_ID {
        Some(AdapterVaultSeeds {
            vault_state: DRIFT_VAULT_STATE_SEED,
            vault_authority: DRIFT_VAULT_AUTHORITY_SEED,
        })
    } else {
        None
    }
}

pub fn is_supported_adapter(adapter: &Address) -> bool {
    vault_seeds(adapter).is_some()
}

pub fn expected_adapter_entry(adapter_program: &Address) -> Result<(Address, u8), ProgramError> {
    based_try_find_program_address(
        &[REGISTRY_ADAPTER_ENTRY_SEED, adapter_program.as_ref()],
        &REGISTRY_PROGRAM_ID,
    )
}

pub fn verify_adapter_entry(
    view: &AccountView,
    adapter_program: &Address,
) -> Result<(), ProgramError> {
    let (expected, _) = expected_adapter_entry(adapter_program)?;
    require_keys_eq!(*view.address(), expected, DispatcherError::AdapterNotApproved);
    require_keys_eq!(
        *view.owner(),
        REGISTRY_PROGRAM_ID,
        DispatcherError::AdapterNotApproved
    );
    let data = view.try_borrow()?;
    adapter_entry_is_approved(&data).map_err(|_| DispatcherError::AdapterNotApproved.into())
}

pub fn adapter_entry_underlying_mint(view: &AccountView) -> Result<Address, ProgramError> {
    let data = view.try_borrow()?;
    read_adapter_entry_underlying_mint(&data).map_err(|_| DispatcherError::AdapterCpiError.into())
}

pub fn is_adapter_vault_state(view: &AccountView, adapter: &Address) -> bool {
    let Some(seeds) = vault_seeds(adapter) else {
        return false;
    };
    if *view.owner() != *adapter {
        return false;
    }
    based_try_find_program_address(&[seeds.vault_state], adapter)
        .map(|(expected, _)| *view.address() == expected)
        .unwrap_or(false)
}

pub fn is_adapter_vault_authority(view: &AccountView, adapter: &Address) -> bool {
    let Some(seeds) = vault_seeds(adapter) else {
        return false;
    };
    based_try_find_program_address(&[seeds.vault_authority], adapter)
        .map(|(expected, _)| *view.address() == expected)
        .unwrap_or(false)
}

pub fn is_adapter_user_position_pda(
    view: &AccountView,
    adapter: &Address,
    user: &Address,
) -> bool {
    if !is_supported_adapter(adapter) {
        return false;
    }
    based_try_find_program_address(&[ADAPTER_POSITION_SEED, user.as_ref()], adapter)
        .map(|(expected, _)| *view.address() == expected)
        .unwrap_or(false)
}

/// Validates an initialized adapter position account (post-deposit).
pub fn is_adapter_user_position(view: &AccountView, adapter: &Address, user: &Address) -> bool {
    is_adapter_user_position_pda(view, adapter, user) && *view.owner() == *adapter
}
