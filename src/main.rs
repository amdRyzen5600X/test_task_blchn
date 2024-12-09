use std::str::FromStr;

use shared_crypto::intent::Intent;
use sui_config::{sui_config_dir, SUI_KEYSTORE_FILENAME};
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};
use sui_sdk::{
    rpc_types::SuiTransactionBlockResponseOptions,
    types::{
        base_types::{ObjectID, SuiAddress},
        clock::Clock,
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        quorum_driver_types::ExecuteTransactionRequestType,
        transaction::{Argument, CallArg, Command, ObjectArg, Transaction, TransactionData},
        Identifier, SUI_CLOCK_OBJECT_ID,
    },
    SuiClientBuilder,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let builder = SuiClientBuilder::default();
    let client = builder.build_testnet().await?;
    let package = ObjectID::from_hex_literal(
        "0x0c7ae833c220aa73a3643a0d508afa4ac5d50d97312ea4584e35f9eb21b9df12",
    )
    .expect("unnable to create ObjectID from hex litteral");

    let mut ptb = ProgrammableTransactionBuilder::new();
    let config = CallArg::Pure(
        bcs::to_bytes("0x9774e359588ead122af1c7e7f64e14ade261cfeecdb5d0eb4a5b3b4c8ab8bd3e")
            .unwrap(),
    );
    ptb.input(config)?;
    let pools = CallArg::Pure(
        bcs::to_bytes("0x50eb61dd5928cec5ea04711a2e9b72e5237e79e9fbcd2ce3d5469dc8708e0ee2")
            .unwrap(),
    );
    ptb.input(pools)?;
    let tick_rate = CallArg::Pure(bcs::to_bytes(&60u32).unwrap());
    ptb.input(tick_rate)?;
    let initialize_price = CallArg::Pure(bcs::to_bytes(&0u128).unwrap());
    ptb.input(initialize_price)?;
    let url = CallArg::Pure(vec![0]);
    ptb.input(url)?;
    let tick_lower = CallArg::Pure(bcs::to_bytes(&0u32).unwrap());
    ptb.input(tick_lower)?;
    let _ = test_task::retrieve_wallet();

    let adress =
        SuiAddress::from_str("0x26c25b83e42a5dd4af29b28db94eacc2caa037d6f311d17b8d7975f50ce4a451")
            .unwrap();
    let coins = client
        .coin_read_api()
        .get_coins(adress, None, None, None)
        .await?;
    let coin_a = coins.data.into_iter().next().unwrap();
    ptb.input(CallArg::Object(ObjectArg::Receiving(coin_a.object_ref())))?;
    let coin_b = coin_a.clone();
    ptb.input(CallArg::Object(ObjectArg::Receiving(coin_b.object_ref())))?;
    let a_meta = client
        .coin_read_api()
        .get_coin_metadata(coin_a.coin_type.clone())
        .await?;
    ptb.input(CallArg::Pure(bcs::to_bytes(&a_meta.unwrap()).unwrap()))?;
    let b_meta = client
        .coin_read_api()
        .get_coin_metadata(coin_b.coin_type)
        .await?;
    ptb.input(CallArg::Pure(bcs::to_bytes(&b_meta.unwrap()).unwrap()))?;
    // let fix_amount_a = ptb.pure(true)?;
    ptb.input(CallArg::Pure(bcs::to_bytes(&true).unwrap()))?;
    let clock = Clock {
        id: sui_sdk::types::id::UID {
            id: sui_sdk::types::id::ID {
                bytes: SUI_CLOCK_OBJECT_ID,
            },
        },
        timestamp_ms: 0,
    };
    ptb.input(CallArg::Pure(bcs::to_bytes(&clock).unwrap()))?;

    ptb.command(Command::move_call(
        package,
        Identifier::new("pool_creator").unwrap(),
        Identifier::new("create_pool_v2").unwrap(),
        vec![],
        vec![
            Argument::Input(0),
            Argument::Input(1),
            Argument::Input(2),
            Argument::Input(3),
            Argument::Input(4),
            Argument::Input(5),
            Argument::Input(6),
            Argument::Input(7),
            Argument::Input(8),
            Argument::Input(9),
            Argument::Input(10),
            Argument::Input(11),
        ],
    ));

    let builder = ptb.finish();
    let gas_budget = 10_000_000;
    let gas_price = client.read_api().get_reference_gas_price().await?;
    // create the transaction data that will be sent to the network
    let tx_data = TransactionData::new_programmable(
        adress,
        vec![coin_a.object_ref()],
        builder,
        gas_budget,
        gas_price,
    );

    // 4) sign transaction
    let keystore = FileBasedKeystore::new(&sui_config_dir()?.join(SUI_KEYSTORE_FILENAME))?;
    let signature = keystore.sign_secure(&adress, &tx_data, Intent::sui_transaction())?;

    // 5) execute the transaction
    print!("Executing the transaction...");
    let transaction_response = client
        .quorum_driver_api()
        .execute_transaction_block(
            Transaction::from_data(tx_data, vec![signature]),
            SuiTransactionBlockResponseOptions::full_content(),
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;
    println!("{}", transaction_response);

    Ok(())
}
