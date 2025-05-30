use std::time::Duration;

use slog::Logger;

use crate::data::subgraph::Link;
use crate::prelude::{DeploymentHash, Error};
use std::fmt::Debug;

mod arweave;
mod file;
mod ipfs;

pub use arweave::*;
use async_trait::async_trait;
pub use file::*;
pub use ipfs::*;

/// Resolves links to subgraph manifests and resources referenced by them.
#[async_trait]
pub trait LinkResolver: Send + Sync + 'static + Debug {
    /// Updates the timeout used by the resolver.
    fn with_timeout(&self, timeout: Duration) -> Box<dyn LinkResolver>;

    /// Enables infinite retries.
    fn with_retries(&self) -> Box<dyn LinkResolver>;

    /// Fetches the link contents as bytes.
    async fn cat(&self, logger: &Logger, link: &Link) -> Result<Vec<u8>, Error>;

    /// Fetches the IPLD block contents as bytes.
    async fn get_block(&self, logger: &Logger, link: &Link) -> Result<Vec<u8>, Error>;

    /// Creates a new resolver that is scoped to a specific subgraph
    /// This is used by FileLinkResolver to create a new resolver for a specific subgraph
    /// For other resolvers, this method will simply return the current resolver
    /// This is required because paths mentioned in the subgraph manifest are relative paths
    /// and we need a new resolver with the right base directory for the specific subgraph
    fn for_deployment(&self, deployment: DeploymentHash) -> Result<Box<dyn LinkResolver>, Error>;

    /// Read the contents of `link` and deserialize them into a stream of JSON
    /// values. The values must each be on a single line; newlines are significant
    /// as they are used to split the file contents and each line is deserialized
    /// separately.
    async fn json_stream(&self, logger: &Logger, link: &Link) -> Result<JsonValueStream, Error>;
}
