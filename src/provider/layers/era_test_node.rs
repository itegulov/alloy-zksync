use alloy::{
    providers::{Provider, ProviderLayer, RootProvider},
    transports::Transport,
};
use std::{
    marker::PhantomData,
    sync::{Arc, OnceLock},
};
use url::Url;

use crate::{
    network::Zksync,
    node_bindings::{EraTestNode, EraTestNodeInstance},
};

/// A layer that wraps an [`EraTestNode`] config.
///
/// The config will be used to spawn an [`EraTestNodeInstance`] when the layer is applied, or when the
/// user requests any information about the anvil node (e.g. via the [`EraTestNodeLayer::endpoint_url`]
/// method).
#[derive(Debug, Clone, Default)]
pub struct EraTestNodeLayer {
    anvil: EraTestNode,
    instance: OnceLock<Arc<EraTestNodeInstance>>,
}

impl EraTestNodeLayer {
    /// Starts the anvil instance, or gets a reference to the existing instance.
    pub fn instance(&self) -> &Arc<EraTestNodeInstance> {
        self.instance
            .get_or_init(|| Arc::new(self.anvil.clone().spawn()))
    }

    /// Get the instance http endpoint.
    #[doc(alias = "http_endpoint_url")]
    pub fn endpoint_url(&self) -> Url {
        self.instance().endpoint_url()
    }
}

impl From<EraTestNode> for EraTestNodeLayer {
    fn from(anvil: EraTestNode) -> Self {
        Self {
            anvil,
            instance: OnceLock::new(),
        }
    }
}

impl<P, T> ProviderLayer<P, T, Zksync> for EraTestNodeLayer
where
    P: Provider<T, Zksync>,
    T: Transport + Clone,
{
    type Provider = EraTestNodeProvider<P, T>;

    fn layer(&self, inner: P) -> Self::Provider {
        let anvil = self.instance();
        EraTestNodeProvider::new(inner, anvil.clone())
    }
}

/// A provider that wraps an [`EraTestNodeInstance`], preventing the instance from
/// being dropped while the provider is in use.
#[derive(Clone, Debug)]
pub struct EraTestNodeProvider<P, T> {
    inner: P,
    _anvil: Arc<EraTestNodeInstance>,
    _pd: PhantomData<fn() -> T>,
}

impl<P, T> EraTestNodeProvider<P, T>
where
    P: Provider<T, Zksync>,
    T: Transport + Clone,
{
    /// Creates a new `EraTestNodeProvider` with the given inner provider and anvil
    /// instance.
    pub fn new(inner: P, _anvil: Arc<EraTestNodeInstance>) -> Self {
        Self {
            inner,
            _anvil,
            _pd: PhantomData,
        }
    }
}

impl<P, T> Provider<T, Zksync> for EraTestNodeProvider<P, T>
where
    P: Provider<T, Zksync>,
    T: Transport + Clone,
{
    #[inline(always)]
    fn root(&self) -> &RootProvider<T, Zksync> {
        self.inner.root()
    }
}
