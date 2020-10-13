use std::fs;
use std::path::PathBuf;

use byteorder::{ByteOrder, LittleEndian};

use near_chain_configs::Genesis;
use near_primitives::account::Account;
use near_primitives::hash::{hash, CryptoHash};
use near_primitives::state_record::StateRecord;
use near_primitives::types::{AccountId, StateRoot};
use near_store::test_utils::create_tries;
use near_store::{ShardTries, TrieUpdate};
use neard::config::GenesisExt;
use node_runtime::{state_viewer::TrieViewer, Runtime};

pub fn alice_account() -> AccountId {
    "alice.near".to_string()
}
pub fn bob_account() -> AccountId {
    "bob.near".to_string()
}
pub fn eve_dot_alice_account() -> AccountId {
    "eve.alice.near".to_string()
}

pub fn default_code_hash() -> CryptoHash {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../runtime/near-vm-runner/tests/res/test_contract_rs.wasm");
    let genesis_wasm = fs::read(path).unwrap();
    hash(&genesis_wasm)
}

const DEFAULT_TEST_CONTRACT: &[u8] =
    include_bytes!("../../../runtime/near-vm-runner/tests/res/test_contract_rs.wasm");

lazy_static::lazy_static! {
    static ref DEFAULT_TEST_CONTRACT_HASH: CryptoHash = hash(&DEFAULT_TEST_CONTRACT);
}

pub fn add_test_contract(genesis: &mut Genesis, account_id: &AccountId) {
    let mut is_account_record_found = false;
    for record in genesis.records.as_mut() {
        if let StateRecord::Account { account_id: record_account_id, ref mut account } = record {
            if record_account_id == account_id {
                is_account_record_found = true;
                account.code_hash = *DEFAULT_TEST_CONTRACT_HASH;
            }
        }
    }
    if !is_account_record_found {
        genesis.records.as_mut().push(StateRecord::Account {
            account_id: account_id.clone(),
            account: Account {
                amount: 0,
                locked: 0,
                code_hash: *DEFAULT_TEST_CONTRACT_HASH,
                storage_usage: 0,
            },
        });
    }
    genesis.records.as_mut().push(StateRecord::Contract {
        account_id: account_id.clone(),
        code: DEFAULT_TEST_CONTRACT.to_vec(),
    });
}

pub fn get_runtime_and_trie_from_genesis(genesis: &Genesis) -> (Runtime, ShardTries, StateRoot) {
    let tries = create_tries();
    let runtime = Runtime::new(Box::new(()));
    let (store_update, genesis_root) = runtime.apply_genesis_state(
        tries.clone(),
        0,
        &genesis
            .config
            .validators
            .iter()
            .map(|account_info| {
                (
                    account_info.account_id.clone(),
                    account_info.public_key.clone(),
                    account_info.amount,
                )
            })
            .collect::<Vec<_>>(),
        &genesis.records.as_ref(),
        &genesis.config.runtime_config,
    );
    store_update.commit().unwrap();
    (runtime, tries, genesis_root)
}

pub fn get_runtime_and_trie() -> (Runtime, ShardTries, StateRoot) {
    let mut genesis = Genesis::test(vec![&alice_account(), &bob_account(), "carol.near"], 3);
    add_test_contract(&mut genesis, &AccountId::from("test.contract"));
    get_runtime_and_trie_from_genesis(&genesis)
}

pub fn get_test_trie_viewer() -> (TrieViewer, TrieUpdate) {
    let (_, tries, root) = get_runtime_and_trie();
    let trie_viewer = TrieViewer::new();
    let state_update = tries.new_trie_update(0, root);
    (trie_viewer, state_update)
}

pub fn encode_int(val: i32) -> [u8; 4] {
    let mut tmp = [0u8; 4];
    LittleEndian::write_i32(&mut tmp, val);
    tmp
}
