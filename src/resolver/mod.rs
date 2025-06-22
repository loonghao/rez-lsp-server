//! Dependency resolution implementation for Rez.

mod conflict_detector;
mod resolver_impl;

pub use conflict_detector::ConflictDetector;
pub use resolver_impl::DependencyResolverImpl;

use crate::core::{DependencyResolver, Requirement, ResolvedContext, Result};

/// Resolve a list of package requirements.
pub async fn resolve_requirements(requirements: &[Requirement]) -> Result<ResolvedContext> {
    let resolver = DependencyResolverImpl::new();
    resolver.resolve(requirements).await
}

/// Check if requirements can be satisfied.
pub async fn can_resolve(requirements: &[Requirement]) -> Result<bool> {
    let resolver = DependencyResolverImpl::new();
    resolver.can_resolve(requirements).await
}
