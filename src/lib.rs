// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use linera_sdk::base::{ContractAbi, ServiceAbi};
use serde::{Deserialize, Serialize};

pub struct ApplicationAbi;

impl ContractAbi for ApplicationAbi {
    type Operation = Operation;
    type Response = ();
}

impl ServiceAbi for ApplicationAbi {
    type Query = async_graphql::Request;
    type QueryResponse = async_graphql::Response;
}

/// Operations that the contract accepts.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Operation {
    /// Checks if a blob with the `blob_id` is available on Walrus and if the SHA-256 hash of its
    /// contents is `blob_hash`.
    CheckBlob {
        blob_id: String,
        blob_hash: [u8; 32],
    },
}
