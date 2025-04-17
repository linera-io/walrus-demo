// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg(not(target_arch = "wasm32"))]

use std::env;

use anyhow::{anyhow, ensure};
use linera_sdk::test::{QueryOutcome, TestValidator};
use reqwest::Url;
use sha2::{Digest, Sha256};
use walrus_demo::{ApplicationAbi, Operation};

/// Tests if the service can read a blob from Walrus.
#[test_log::test(tokio::test)]
async fn service_can_read_blob() -> anyhow::Result<()> {
    let aggregator_url = env::var("WALRUS_AGGREGATOR_URL")
        .unwrap_or("https://aggregator.walrus-testnet.walrus.space".to_owned());
    let aggregator_host = Url::parse(&aggregator_url)?
        .host_str()
        .expect("Missing host in the URL")
        .to_owned();

    let (validator, application_id, chain) =
        TestValidator::with_current_application::<ApplicationAbi, _, _>("".to_owned(), ()).await;

    validator
        .change_resource_control_policy({
            |policy| {
                policy.http_request_allow_list.insert(aggregator_host);
            }
        })
        .await;

    let blob_contents = "Linera test blob";
    let blob_id = publish_blob(blob_contents).await?;

    let query = format!(
        "query {{ \
        readBlob(\
            aggregatorUrl: \"{aggregator_url}\", \
            blobId: \"{blob_id}\"\
        ) \
        \
    }}"
    );

    let QueryOutcome { response, .. } = chain.graphql_query(application_id, query).await;
    let read_blob_data = response["readBlob"]
        .as_array()
        .expect("Invalid `readBlob` response from service")
        .iter()
        .map(|byte_value| {
            byte_value
                .as_i64()
                .expect("Invalid byte type")
                .try_into()
                .expect("Invalid byte value")
        })
        .collect::<Vec<u8>>();

    assert_eq!(read_blob_data, blob_contents.as_bytes());

    Ok(())
}

/// Tests if the contract can read and check a blob from Walrus.
#[test_log::test(tokio::test)]
async fn contract_can_check_blob() -> anyhow::Result<()> {
    let aggregator_url = env::var("WALRUS_AGGREGATOR_URL")
        .unwrap_or("https://aggregator.walrus-testnet.walrus.space".to_owned());
    let aggregator_host = Url::parse(&aggregator_url)?
        .host_str()
        .expect("Missing host in the URL")
        .to_owned();

    let (validator, application_id, chain) = TestValidator::with_current_application::<
        ApplicationAbi,
        _,
        _,
    >(aggregator_url.to_owned(), ())
    .await;

    validator
        .change_resource_control_policy({
            |policy| {
                policy.http_request_allow_list.insert(aggregator_host);
            }
        })
        .await;

    let blob_contents = "Linera test blob";
    let blob_id = publish_blob(blob_contents).await?;
    let blob_hash = Sha256::digest(blob_contents.as_bytes()).into();

    chain
        .add_block(|block| {
            block.with_operation(application_id, Operation::CheckBlob { blob_id, blob_hash });
        })
        .await;

    Ok(())
}

/// Uses a public Walrus publisher to publish a blob with the provided `blob_contents`.
///
/// Returns the blob's identifier.
async fn publish_blob(blob_contents: &'static str) -> anyhow::Result<String> {
    let publisher_url = env::var("WALRUS_PUBLISHER_URL")
        .unwrap_or("https://publisher.walrus-testnet.walrus.space".to_owned());

    let http_client = reqwest::Client::new();
    let publish_response = http_client
        .put(format!("{publisher_url}/v1/blobs"))
        .body(blob_contents)
        .send()
        .await?;

    ensure!(
        publish_response.status().is_success(),
        anyhow!("Failed to publish blob. Publisher response: {publish_response:#?}")
    );

    let blob_metadata = publish_response.json::<serde_json::Value>().await?;

    let blob_id_value =
        if let serde_json::Value::Object(already_certified) = &blob_metadata["alreadyCertified"] {
            &already_certified["blobId"]
        } else if let serde_json::Value::Object(newly_created) = &blob_metadata["newlyCreated"] {
            &newly_created["blobObject"]["blobId"]
        } else {
            anyhow::bail!("Unexpected response from publisher: {blob_metadata}");
        };

    match blob_id_value {
        serde_json::Value::String(blob_id) => Ok(blob_id.clone()),
        _ => Err(anyhow!("Missing blob ID in response: {blob_metadata}")),
    }
}
