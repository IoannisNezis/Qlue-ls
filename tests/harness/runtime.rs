//! Async runtime helpers for LSP tests
//!
//! The LSP server uses Tokio's LocalSet for task spawning, so tests must
//! run within a properly configured async runtime.

use std::future::Future;

/// Run an async LSP test with the proper Tokio runtime configuration.
///
/// This function sets up a single-threaded Tokio runtime with a LocalSet,
/// which is required by the LSP server's async task spawning.
///
/// # Example
/// ```ignore
/// #[test]
/// fn test_something() {
///     run_lsp_test(|| async {
///         let client = TestClient::new();
///         client.initialize().await;
///         // ... test logic
///     });
/// }
/// ```
pub fn run_lsp_test<F, Fut>(test: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    let local = tokio::task::LocalSet::new();
    rt.block_on(local.run_until(test()));
}

/// Macro for defining LSP tests with proper async runtime setup.
///
/// This macro creates a test function that runs the provided async block
/// within a properly configured Tokio runtime with LocalSet support.
///
/// # Example
/// ```ignore
/// lsp_test!(test_initialize, {
///     let client = TestClient::new();
///     let id = client.initialize().await;
///     let response = client.get_response(id).unwrap();
///     assert!(response["result"]["capabilities"].is_object());
/// });
/// ```
#[macro_export]
macro_rules! lsp_test {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            $crate::harness::runtime::run_lsp_test(|| async { $body });
        }
    };
}
