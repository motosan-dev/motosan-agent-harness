#![warn(missing_docs)]
//! # motosan-agent-harness
//!
//! Composition contract for the Motosan agent framework. This crate owns the
//! single trait [`Harness`] — the vertical bundle that ties a set of tools,
//! hooks, a permission policy, a memory schema, and a system prompt into one
//! coherent domain (finance, rental, devops, …).
//!
//! It is deliberately a thin crate: no agent loop, no registry, no runtime.
//! Loop crates depend on this trait; harness implementations depend on this
//! trait; nobody else needs to.
//!
//! ## Layering
//!
//! ```text
//!    ┌────────────────────────────────────────────┐
//!    │ motosan-agent-harness-{finance,rental,…}   │  vertical implementations
//!    ├────────────────────────────────────────────┤
//!    │ motosan-agent-loop      (ReAct engine)     │  runs Harness + Tools
//!    ├────────────────────────────────────────────┤
//!    │ motosan-agent-harness   (Harness trait)    │  THIS CRATE
//!    ├────────────────────────────────────────────┤
//!    │ motosan-agent-tool │ motosan-ai │ sandbox  │  capability + infra
//!    ├────────────────────────────────────────────┤
//!    │ motosan-agent-primitives                   │  shared types + Hook + Permission
//!    └────────────────────────────────────────────┘
//! ```
//!
//! ## Why a separate crate?
//!
//! Per decision **D1=B** in the primitives plan, the `Tool` trait stays in
//! `motosan-agent-tool` and the `Harness` trait lives here. That keeps
//! `motosan-agent-primitives` a leaf contract layer with no dependency on
//! `Tool`, while still giving downstream loop / runner crates a single trait
//! to plug verticals into.
//!
//! ## Stability
//!
//! `0.x` — the trait surface is iterating. The API will be frozen at `1.0`
//! once at least two real harnesses (finance + rental) have been built
//! against it and the awkwardness list has been resolved.

pub mod harness;

pub use harness::Harness;
