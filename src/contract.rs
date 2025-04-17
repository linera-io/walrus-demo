// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use linera_sdk::{
    http,
    linera_base_types::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};
use sha2::{Digest, Sha256};
use walrus_demo::Operation;

use self::state::Application;

pub struct ApplicationContract {
    state: Application,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(ApplicationContract);

impl WithContractAbi for ApplicationContract {
    type Abi = walrus_demo::ApplicationAbi;
}

impl Contract for ApplicationContract {
    type Message = ();
    type EventValue = ();
    type Parameters = String;
    type InstantiationArgument = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = Application::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        ApplicationContract { state, runtime }
    }

    async fn instantiate(&mut self, _argument: Self::InstantiationArgument) {}

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        let Operation::CheckBlob { blob_id, blob_hash } = operation;

        let aggregator_url = self.runtime.application_parameters();
        let response = self.runtime.http_request(http::Request::get(format!(
            "{aggregator_url}/v1/blobs/{blob_id}"
        )));

        assert_eq!(
            response.status, 200,
            "Failed to read blob. Status code: {}",
            response.status
        );

        let calculated_hash = Sha256::digest(response.body);

        assert_eq!(
            calculated_hash,
            blob_hash.into(),
            "Expected blob hash {}, but got {}",
            hex::encode(blob_hash),
            hex::encode(calculated_hash)
        );
    }

    async fn execute_message(&mut self, _message: Self::Message) {}

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}
