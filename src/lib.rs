//! ga-core - Core engine for GenericAgent
//!
//! This crate replaces Python's agent_loop.py, ga.py, and llmcore.py
//! with a Rust-based core engine exposing an HTTP API via axum.

pub mod agent;
pub mod api;
pub mod browser;
pub mod cli;
pub mod config;
pub mod llm;
pub mod tools;
pub mod utils;
pub mod core;
