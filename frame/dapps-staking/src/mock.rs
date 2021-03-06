//! Test utilities

#![cfg(test)]

use super::*;
use frame_support::{
    assert_ok, impl_outer_dispatch, impl_outer_origin, parameter_types,
    traits::OnFinalize,
    weights::{WeightToFeeCoefficients, WeightToFeePolynomial},
};
use pallet_contracts::Gas;
use pallet_plasm_rewards::{inflation::SimpleComputeTotalPayout, traits::MaybeValidators};
use sp_core::{crypto::key_types, H256};
use sp_runtime::{
    testing::{Header, UintAuthorityId},
    traits::{BlakeTwo256, ConvertInto, Hash, IdentityLookup, OpaqueKeys},
    KeyTypeId, Perbill,
};

pub type BlockNumber = u64;
pub type AccountId = u64;
pub type Balance = u64;

pub const ALICE_STASH: u64 = 1;
pub const BOB_STASH: u64 = 2;
pub const ALICE_CTRL: u64 = 3;
pub const BOB_CTRL: u64 = 4;
pub const VALIDATOR_A: u64 = 5;
pub const VALIDATOR_B: u64 = 6;
pub const OPERATOR_A: u64 = 9;
pub const OPERATOR_B: u64 = 10;
pub const OPERATED_CONTRACT_A: u64 = 19;
pub const OPERATED_CONTRACT_B: u64 = 20;
pub const BOB_CONTRACT: u64 = 12;

impl_outer_origin! {
    pub enum Origin for Test {}
}

impl_outer_dispatch! {
    pub enum Call for Test where origin: Origin {
        pallet_session::Session,
        pallet_balances::Balances,
        pallet_contracts::Contracts,
        dapps_staking::DappsStaking,
    }
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let _ = pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE_STASH, 1000),
            (BOB_STASH, 2000),
            (ALICE_CTRL, 10),
            (BOB_CTRL, 20),
            (VALIDATOR_A, 1_000_000),
            (VALIDATOR_B, 1_000_000),
        ],
    }
    .assimilate_storage(&mut storage);

    let _ = pallet_contracts::GenesisConfig {
        current_schedule: pallet_contracts::Schedule {
            enable_println: true,
            ..Default::default()
        },
    }
    .assimilate_storage(&mut storage);

    let _ = pallet_plasm_rewards::GenesisConfig {
        ..Default::default()
    }
    .assimilate_storage(&mut storage);

    let _ = GenesisConfig {
        ..Default::default()
    }
    .assimilate_storage(&mut storage);

    let validators = vec![VALIDATOR_A, VALIDATOR_B];

    let _ = pallet_session::GenesisConfig::<Test> {
        keys: validators
            .iter()
            .map(|x| (*x, *x, UintAuthorityId(*x)))
            .collect(),
    }
    .assimilate_storage(&mut storage);

    storage.into()
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: u32 = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl system::Trait for Test {
    type Origin = Origin;
    type BaseCallFilter = ();
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = ();
    type SystemWeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 1;
}
impl pallet_timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const Period: u64 = 1;
    pub const Offset: u64 = 0;
}

pub struct TestSessionHandler;

impl pallet_session::SessionHandler<u64> for TestSessionHandler {
    const KEY_TYPE_IDS: &'static [KeyTypeId] = &[key_types::DUMMY];
    fn on_genesis_session<T: OpaqueKeys>(_validators: &[(u64, T)]) {}
    fn on_new_session<T: OpaqueKeys>(
        _changed: bool,
        _validators: &[(u64, T)],
        _queued_validators: &[(u64, T)],
    ) {
    }
    fn on_disabled(_validator_index: usize) {}
    fn on_before_session_ending() {}
}

impl pallet_session::Trait for Test {
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = PlasmRewards;
    type SessionHandler = TestSessionHandler;
    type ValidatorId = u64;
    type ValidatorIdOf = ConvertInto;
    type Keys = UintAuthorityId;
    type Event = ();
    type DisabledValidatorsThreshold = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 10;
}

impl pallet_balances::Trait for Test {
    type Balance = Balance;
    type Event = ();
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = system::Module<Test>;
    type WeightInfo = ();
    type MaxLocks = ();
}

pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
    type Balance = u64;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        Default::default()
    }
}

parameter_types! {
    pub const TransactionByteFee: u64 = 0;
}

impl pallet_transaction_payment::Trait for Test {
    type Currency = Balances;
    type OnTransactionPayment = ();
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = WeightToFee;
    type FeeMultiplierUpdate = ();
}

pub struct DummyContractAddressFor;
impl pallet_contracts::ContractAddressFor<H256, u64> for DummyContractAddressFor {
    fn contract_address_for(_code_hash: &H256, _data: &[u8], origin: &u64) -> u64 {
        *origin + 10
    }
}

pub struct DummyTrieIdGenerator;

impl pallet_contracts::TrieIdGenerator<u64> for DummyTrieIdGenerator {
    fn trie_id(account_id: &u64) -> pallet_contracts::TrieId {
        use sp_core::storage::well_known_keys;

        let new_seed = pallet_contracts::AccountCounter::mutate(|v| {
            *v = v.wrapping_add(1);
            *v
        });

        // TODO: see https://github.com/paritytech/substrate/issues/2325
        let mut res = vec![];
        res.extend_from_slice(well_known_keys::CHILD_STORAGE_KEY_PREFIX);
        res.extend_from_slice(b"default:");
        res.extend_from_slice(&new_seed.to_le_bytes());
        res.extend_from_slice(&account_id.to_le_bytes());
        res
    }
}

parameter_types! {
    pub const ContractTransactionBaseFee: Balance = 0;
    pub const ContractTransactionByteFee: Balance = 0;
    pub const ContractFee: Balance = 0;
    pub const TombstoneDeposit: Balance = 0;
    pub const RentByteFee: Balance = 0;
    pub const RentDepositOffset: Balance = 0;
    pub const SurchargeReward: Balance = 0;
}

impl pallet_contracts::Trait for Test {
    type Time = Timestamp;
    type Randomness = pallet_randomness_collective_flip::Module<Test>;
    type Currency = Balances;
    type Event = ();
    type DetermineContractAddress = DummyContractAddressFor;
    type TrieIdGenerator = DummyTrieIdGenerator;
    type RentPayment = ();
    type SignedClaimHandicap = pallet_contracts::DefaultSignedClaimHandicap;
    type TombstoneDeposit = TombstoneDeposit;
    type StorageSizeOffset = pallet_contracts::DefaultStorageSizeOffset;
    type RentByteFee = RentByteFee;
    type RentDepositOffset = RentDepositOffset;
    type SurchargeReward = SurchargeReward;
    type MaxDepth = pallet_contracts::DefaultMaxDepth;
    type MaxValueSize = pallet_contracts::DefaultMaxValueSize;
    type WeightPrice = pallet_transaction_payment::Module<Self>;
}

impl pallet_contract_operator::Trait for Test {
    type Parameters = parameters::StakingParameters;
    type Event = ();
}

pub struct DummyMaybeValidators;
impl MaybeValidators<EraIndex, AccountId> for DummyMaybeValidators {
    fn compute(_current_era: EraIndex) -> Option<Vec<AccountId>> {
        Some(vec![1, 2, 3])
    }
}

parameter_types! {
    pub const SessionsPerEra: sp_staking::SessionIndex = 10;
    pub const BondingDuration: EraIndex = 3;
}

impl pallet_plasm_rewards::Trait for Test {
    type Currency = Balances;
    type Time = Timestamp;
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type ComputeEraForDapps = DappsStaking;
    type ComputeEraForSecurity = DappsStaking;
    type ComputeTotalPayout = SimpleComputeTotalPayout;
    type MaybeValidators = DummyMaybeValidators;
    type Event = ();
}

impl Trait for Test {
    type Currency = Balances;
    type BondingDuration = BondingDuration;
    type ContractFinder = Operator;
    type RewardRemainder = (); // Reward remainder is burned.
    type Reward = (); // Reward is minted.
    type Time = Timestamp;
    type ComputeRewardsForDapps = rewards::VoidableRewardsForDapps;
    type EraFinder = PlasmRewards;
    type ForDappsEraReward = PlasmRewards;
    type HistoryDepthFinder = PlasmRewards;
    type Event = ();
}

/// ValidatorManager module.
pub type System = system::Module<Test>;
pub type Session = pallet_session::Module<Test>;
pub type Balances = pallet_balances::Module<Test>;
pub type Timestamp = pallet_timestamp::Module<Test>;
pub type Contracts = pallet_contracts::Module<Test>;
pub type Operator = pallet_contract_operator::Module<Test>;
pub type PlasmRewards = pallet_plasm_rewards::Module<Test>;
pub type DappsStaking = Module<Test>;

/// Generate Wasm binary and code hash from wabt source.
pub fn compile_module<T>(
    wabt_module: &str,
) -> result::Result<(Vec<u8>, <T::Hashing as Hash>::Output), wabt::Error>
where
    T: system::Trait,
{
    let wasm = wabt::wat2wasm(wabt_module)?;
    let code_hash = T::Hashing::hash(&wasm);
    Ok((wasm, code_hash))
}

pub const CODE_RETURN_FROM_START_FN: &str = r#"
(module
	(import "seal0" "seal_return" (func $seal_return (param i32 i32 i32)))
	(import "seal0" "seal_deposit_event" (func $seal_deposit_event (param i32 i32 i32 i32)))
	(import "env" "memory" (memory 1 1))

	(start $start)
	(func $start
		(call $seal_deposit_event
			(i32.const 0) ;; The topics buffer
			(i32.const 0) ;; The topics buffer's length
			(i32.const 8) ;; The data buffer
			(i32.const 4) ;; The data buffer's length
		)
		(call $seal_return
			(i32.const 0)
			(i32.const 8)
			(i32.const 4)
		)
		(unreachable)
	)

	(func (export "call")
		(unreachable)
	)
	(func (export "deploy"))

	(data (i32.const 8) "\01\02\03\04")
)
"#;

pub const CODE_RETURN_FROM_START_FN_B: &str = CODE_RETURN_FROM_START_FN;

pub fn valid_instatiate() {
    let (wasm, code_hash) = compile_module::<Test>(CODE_RETURN_FROM_START_FN).unwrap();

    let (wasm_b, code_hash_b) = compile_module::<Test>(CODE_RETURN_FROM_START_FN_B).unwrap();

    // prepare
    let _ = Balances::deposit_creating(&OPERATOR_A, 1_000_000);
    assert_ok!(Contracts::put_code(Origin::signed(OPERATOR_A), wasm));

    let _ = Balances::deposit_creating(&OPERATOR_B, 1_000_000);
    assert_ok!(Contracts::put_code(Origin::signed(OPERATOR_B), wasm_b));

    let test_params = parameters::StakingParameters {
        can_be_nominated: true,
        option_expired: 100,
        option_p: Perbill::from_percent(20).deconstruct(),
    };

    // instantiate
    // Check at the end to get hash on error easily
    let _ = Operator::instantiate(
        Origin::signed(OPERATOR_A),
        100,
        Gas::max_value(),
        code_hash.into(),
        vec![],
        test_params.clone(),
    );
    let _ = Operator::instantiate(
        Origin::signed(OPERATOR_B),
        100,
        Gas::max_value(),
        code_hash_b.into(),
        vec![],
        test_params.clone(),
    );

    // checks deployed contract
    assert!(pallet_contracts::ContractInfoOf::<Test>::contains_key(
        OPERATED_CONTRACT_A
    ));
    assert!(pallet_contracts::ContractInfoOf::<Test>::contains_key(
        OPERATED_CONTRACT_B
    ));

    // checks mapping operator and contract
    // OPERATOR_A operates a only OPERATED_CONTRACT_A contract.
    assert!(pallet_contract_operator::OperatorHasContracts::<Test>::contains_key(OPERATOR_A));
    let tree = pallet_contract_operator::OperatorHasContracts::<Test>::get(&OPERATOR_A);
    assert_eq!(tree.len(), 1);
    assert!(tree.contains(&OPERATED_CONTRACT_A));

    // checks mapping operator and contract
    // OPERATOR_B operates a only OPERATED_CONTRACT_B contract.
    assert!(pallet_contract_operator::OperatorHasContracts::<Test>::contains_key(OPERATOR_B));
    let tree = pallet_contract_operator::OperatorHasContracts::<Test>::get(&OPERATOR_B);
    assert_eq!(tree.len(), 1);
    assert!(tree.contains(&OPERATED_CONTRACT_B));

    // OPERATED_CONTRACT_A contract is operated by OPERATOR_A.
    assert!(
        pallet_contract_operator::ContractHasOperator::<Test>::contains_key(OPERATED_CONTRACT_A)
    );
    assert_eq!(
        pallet_contract_operator::ContractHasOperator::<Test>::get(&OPERATED_CONTRACT_A),
        Some(OPERATOR_A)
    );

    // OPERATED_CONTRACT_B contract is operated by OPERATOR_B.
    assert!(
        pallet_contract_operator::ContractHasOperator::<Test>::contains_key(OPERATED_CONTRACT_B)
    );
    assert_eq!(
        pallet_contract_operator::ContractHasOperator::<Test>::get(&OPERATED_CONTRACT_B),
        Some(OPERATOR_B)
    );

    // OPERATED_CONTRACT's contract Parameters is same test_params.
    assert!(
        pallet_contract_operator::ContractParameters::<Test>::contains_key(OPERATED_CONTRACT_A)
    );
    assert_eq!(
        pallet_contract_operator::ContractParameters::<Test>::get(&OPERATED_CONTRACT_A),
        Some(test_params.clone())
    );

    // OPERATED_CONTRACT_B's contract Parameters is same test_params.
    assert!(
        pallet_contract_operator::ContractParameters::<Test>::contains_key(OPERATED_CONTRACT_B)
    );
    assert_eq!(
        pallet_contract_operator::ContractParameters::<Test>::get(&OPERATED_CONTRACT_B),
        Some(test_params)
    );
}

pub const PER_SESSION: u64 = 60 * 1000;

pub fn advance_session() {
    // increase block numebr
    let next = System::block_number() + 1;
    System::set_block_number(next);
    // increase timestamp + 10
    let now_time = Timestamp::get();
    Timestamp::set_timestamp(now_time + PER_SESSION);
    Session::rotate_session();
    assert_eq!(Session::current_index(), (next / Period::get()) as u32);

    // on finalize
    PlasmRewards::on_finalize(next);
}

pub fn advance_era() {
    let current_era = PlasmRewards::current_era().unwrap();
    while current_era == PlasmRewards::current_era().unwrap() {
        advance_session();
    }
}
