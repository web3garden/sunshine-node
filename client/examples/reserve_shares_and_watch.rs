use sp_keyring::AccountKeyring;
use sunshine_client::shares_atomic::{reserve_shares, SharesAtomic};
use sunshine_client::system::System;
use sunshine_client::{ClientBuilder, ExtrinsicSuccess, Runtime};

type AccountId = <Runtime as System>::AccountId;
type OrgId = <Runtime as SharesAtomic>::OrgId;
type ShareId = <Runtime as SharesAtomic>::ShareId;

fn main() {
    let result: Result<ExtrinsicSuccess<_>, Box<dyn std::error::Error + 'static>> =
        async_std::task::block_on(async move {
            env_logger::init();

            let alice_the_signer = AccountKeyring::Alice.pair();

            let reserves_alices_shares = AccountKeyring::Alice.to_account_id();

            let organization: OrgId = 1u64;
            let share_id: ShareId = 1u64;

            let cli = ClientBuilder::new().build().await?;
            let xt = cli.xt(alice_the_signer, None).await?;
            let xt_result = xt
                .watch()
                .events_decoder(|decoder| {
                    // for any primitive event with no type size registered
                    decoder.register_type_size::<(u64, u64, u64)>("IdentificationTuple")
                })
                .submit(reserve_shares::<Runtime>(
                    organization,
                    share_id,
                    reserves_alices_shares.clone().into(),
                ))
                .await?;
            Ok(xt_result)
        });
    match result {
        Ok(extrinsic_success) => {
            match extrinsic_success
                .find_event::<(OrgId, ShareId, AccountId, u32)>("SharesAtomic", "SharesReserved")
            {
                Some(Ok((org, share, account, amt))) => println!(
                    "Account {:?} reserved {:?} shares with share id {:?} for organization id {:?}",
                    account, amt, share, org
                ),
                Some(Err(err)) => println!("Failed to decode code hash: {}", err),
                None => println!("Failed to find SharesAtomic::Reserve Event"),
            }
        }
        Err(err) => println!("Error: {:?}", err),
    }
}