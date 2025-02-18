// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use linera_sdk::base::{ContractAbi, ServiceAbi};

pub struct ApplicationAbi;

impl ContractAbi for ApplicationAbi {
    type Operation = ();
    type Response = ();
}

impl ServiceAbi for ApplicationAbi {
    type Query = async_graphql::Request;
    type QueryResponse = async_graphql::Response;
}
