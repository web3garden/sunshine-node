use libipld::{
    cache::{
        CacheConfig,
        IpldCache,
    },
    cbor::DagCborCodec,
    derive_cache,
    store::Store,
};
use substrate_subxt::{
    balances::{
        AccountData,
        Balances,
    },
    extrinsic,
    sp_core,
    sp_runtime,
    sp_runtime::traits::{
        IdentifyAccount,
        Verify,
    },
    system::System,
};
use sunshine_bounty_client::{
    bank::Bank,
    bounty::Bounty,
    donate::Donate,
    org::Org,
    vote::Vote,
};
use sunshine_client_utils::{
    client::{
        GenericClient,
        KeystoreImpl,
        OffchainStoreImpl,
    },
    codec::hasher::BLAKE2B_256,
    crypto::{
        keychain::KeyType,
        sr25519,
    },
    node::{
        ChainSpecError,
        Configuration,
        NodeConfig,
        RpcHandlers,
        ScServiceError,
        TaskManager,
    },
};

pub use sunshine_bounty_client::*;
pub use sunshine_bounty_utils as utils;
pub use sunshine_client_utils as client;

pub type AccountId = <<sp_runtime::MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Uid = u32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Runtime;

impl System for Runtime {
    type Index = u32;
    type BlockNumber = u32;
    type Hash = sp_core::H256;
    type Hashing = sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Address = AccountId;
    type Header = sp_runtime::generic::Header<Self::BlockNumber, Self::Hashing>;
    type Extrinsic = sp_runtime::OpaqueExtrinsic;
    type AccountData = AccountData<u128>;
}

impl Balances for Runtime {
    type Balance = u128;
}

impl Org for Runtime {
    type Cid = u32;
    type OrgId = u64;
    type Shares = u64;
    type Constitution = TextBlock;
}

impl Vote for Runtime {
    type VoteId = u64;
    type Signal = u64;
    type ThresholdId = u64;
    type Percent = sp_runtime::Permill;
    type VoteTopic = TextBlock;
    type VoterView = utils::vote::VoterView;
    type VoteJustification = TextBlock;
}

impl Donate for Runtime {}

impl Bank for Runtime {
    type BankId = u64;
    type SpendId = u64;
}

impl Bounty for Runtime {
    type IpfsReference = u32;
    type BountyId = u64;
    type BountyPost = GithubIssue;
    type SubmissionId = u64;
    type BountySubmission = GithubIssue;
}

impl sunshine_identity_client::Identity for Runtime {
    type Uid = u8;
    type Cid = sunshine_codec::Cid;
    type Mask = u8;
    type Gen = u16;
    type IdAccountData = ();
}

impl sunshine_faucet_client::Faucet for Runtime {}

impl substrate_subxt::Runtime for Runtime {
    type Signature = sp_runtime::MultiSignature;
    type Extra = extrinsic::DefaultExtra<Self>;
}

pub struct OffchainClient<S> {
    bounties: IpldCache<S, DagCborCodec, GithubIssue>,
    constitutions: IpldCache<S, DagCborCodec, TextBlock>,
}

impl<S: Store> OffchainClient<S> {
    pub fn new(store: S) -> Self {
        let (mut config, mut config2) = (
            CacheConfig::new(store.clone(), DagCborCodec),
            CacheConfig::new(store, DagCborCodec),
        );
        config.size = 64;
        config2.size = 64;
        config.hash = BLAKE2B_256;
        config2.hash = BLAKE2B_256;
        Self {
            bounties: IpldCache::new(config),
            constitutions: IpldCache::new(config2),
        }
    }
}

derive_cache!(OffchainClient, bounties, DagCborCodec, GithubIssue);
derive_cache!(OffchainClient, constitutions, DagCborCodec, TextBlock);

impl<S: Store> From<S> for OffchainClient<S> {
    fn from(store: S) -> Self {
        Self::new(store)
    }
}

pub struct Node;

impl NodeConfig for Node {
    type ChainSpec = test_node::ChainSpec;
    type Runtime = Runtime;

    fn impl_name() -> &'static str {
        test_node::IMPL_NAME
    }

    fn impl_version() -> &'static str {
        test_node::IMPL_VERSION
    }

    fn author() -> &'static str {
        test_node::AUTHOR
    }

    fn copyright_start_year() -> i32 {
        test_node::COPYRIGHT_START_YEAR
    }

    fn chain_spec_dev() -> Self::ChainSpec {
        test_node::development_config()
    }

    fn chain_spec_from_json_bytes(
        json: Vec<u8>,
    ) -> Result<Self::ChainSpec, ChainSpecError> {
        Self::ChainSpec::from_json_bytes(json).map_err(ChainSpecError)
    }

    fn new_light(
        config: Configuration,
    ) -> Result<(TaskManager, RpcHandlers), ScServiceError> {
        Ok(test_node::new_light(config)?)
    }

    fn new_full(
        config: Configuration,
    ) -> Result<(TaskManager, RpcHandlers), ScServiceError> {
        Ok(test_node::new_full(config)?)
    }
}

pub struct UserDevice;

impl KeyType for UserDevice {
    const KEY_TYPE: u8 = 0;
    type Pair = sr25519::Pair;
}

pub type Client = GenericClient<
    Node,
    UserDevice,
    KeystoreImpl<UserDevice>,
    OffchainClient<OffchainStoreImpl>,
>;

#[cfg(feature = "mock")]
pub mod mock {
    use super::*;
    use sunshine_client_utils::mock::{
        self,
        build_test_node,
        OffchainStoreImpl,
    };
    pub use sunshine_client_utils::mock::{
        AccountKeyring,
        TempDir,
        TestNode,
    };

    pub type Client = GenericClient<
        Node,
        UserDevice,
        mock::KeystoreImpl<UserDevice>,
        OffchainClient<OffchainStoreImpl>,
    >;

    pub type ClientWithKeystore = GenericClient<
        Node,
        UserDevice,
        KeystoreImpl<UserDevice>,
        OffchainClient<OffchainStoreImpl>,
    >;

    pub fn test_node() -> (TestNode, TempDir) {
        build_test_node::<Node>()
    }
}
