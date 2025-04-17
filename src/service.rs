// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::sync::Arc;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use linera_sdk::{ensure, http, linera_base_types::WithServiceAbi, Service, ServiceRuntime};

#[derive(Clone)]
pub struct ApplicationService {
    runtime: Arc<ServiceRuntime<Self>>,
}

linera_sdk::service!(ApplicationService);

impl WithServiceAbi for ApplicationService {
    type Abi = walrus_demo::ApplicationAbi;
}

impl Service for ApplicationService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        ApplicationService {
            runtime: Arc::new(runtime),
        }
    }

    async fn handle_query(&self, query: Self::Query) -> Self::QueryResponse {
        Schema::build(
            Query {
                runtime: self.runtime.clone(),
            },
            EmptyMutation,
            EmptySubscription,
        )
        .finish()
        .execute(query)
        .await
    }
}

/// Root type that defines all the GraphQL queries available from the service.
pub struct Query {
    runtime: Arc<ServiceRuntime<ApplicationService>>,
}

#[async_graphql::Object]
impl Query {
    /// Reads a blob from Walrus.
    async fn read_blob(
        &self,
        aggregator_url: String,
        blob_id: String,
    ) -> async_graphql::Result<Vec<u8>> {
        let response = self.runtime.http_request(http::Request::get(format!(
            "{aggregator_url}/v1/blobs/{blob_id}"
        )));

        ensure!(
            response.status == 200,
            async_graphql::Error::new(format!(
                "Failed to read blob. Status code: {}",
                response.status
            ))
        );

        Ok(response.body)
    }
}
