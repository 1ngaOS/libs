#![cfg(feature = "graphql-api")]

use std::sync::Arc;

use async_graphql::{Context, Object, Schema};
use async_graphql_axum::GraphQL;
use axum::{routing::post, Router};

use crate::{
    build_once_off_checkout, build_subscription_checkout, AdvancedPaymentRequest,
    CheckoutResponse, OnceOffPaymentRequest, PayFastConfig,
};

/// GraphQL schema root Query type (currently minimal).
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn api_version(&self) -> &str {
        "1.0"
    }
}

/// GraphQL schema root Mutation type exposing PayFast operations.
pub struct MutationRoot {
    cfg: Arc<GraphqlConfig>,
}

impl MutationRoot {
    fn new(cfg: Arc<GraphqlConfig>) -> Self {
        Self { cfg }
    }
}

#[Object]
impl MutationRoot {
    /// Create a once-off PayFast checkout (returns URL + params).
    async fn create_once_off_payment(
        &self,
        _ctx: &Context<'_>,
        input: OnceOffPaymentRequest,
    ) -> CheckoutResponse {
        build_once_off_checkout(&self.cfg.payfast, self.cfg.sandbox, input)
    }

    /// Create a subscription / split payment checkout.
    async fn create_subscription_payment(
        &self,
        _ctx: &Context<'_>,
        input: AdvancedPaymentRequest,
    ) -> CheckoutResponse {
        build_subscription_checkout(&self.cfg.payfast, self.cfg.sandbox, input)
    }
}

/// Shared configuration for the GraphQL API.
#[derive(Clone)]
pub struct GraphqlConfig {
    /// Core PayFast configuration (merchant id/key, passphrase).
    pub payfast: PayFastConfig,
    /// Whether to use the sandbox (`true`) or live gateway (`false`).
    pub sandbox: bool,
}

/// Build an `axum::Router` exposing a GraphQL endpoint at `/graphql`.
pub fn graphql_router(cfg: GraphqlConfig) -> Router {
    let cfg = Arc::new(cfg);
    let schema = Schema::build(QueryRoot, MutationRoot::new(cfg.clone()), ())
        .data(cfg)
        .finish();

    let graphql = GraphQL::new(schema);

    Router::new().route("/graphql", post(graphql))
}

