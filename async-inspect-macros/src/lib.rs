//! Procedural macros for async-inspect
//!
//! This crate provides attribute macros for automatic instrumentation of async functions.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, visit_mut::VisitMut, Expr, ItemFn};

/// Attribute macro to automatically instrument async functions
///
/// # Example
///
/// ```rust,ignore
/// #[async_inspect::trace]
/// async fn fetch_user(id: u64) -> User {
///     let profile = fetch_profile(id).await;  // Automatically tracked!
///     let posts = fetch_posts(id).await;      // Automatically tracked!
///     User { profile, posts }
/// }
/// ```
///
/// This macro will:
/// - Register the function as a tracked task
/// - Automatically label each `.await` point
/// - Track execution time
/// - Report completion or failure
#[proc_macro_attribute]
pub fn trace(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);

    // Ensure it's an async function
    if input.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            input.sig.fn_token,
            "#[async_inspect::trace] can only be applied to async functions",
        )
        .to_compile_error()
        .into();
    }

    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &input.vis;
    let sig = &input.sig;

    // Instrument the function body
    let mut instrumenter = AwaitInstrumenter {
        counter: 0,
        fn_name: fn_name_str.clone(),
    };
    instrumenter.visit_block_mut(&mut input.block);

    let instrumented_block = &input.block;

    let output = quote! {
        #vis #sig {
            // Register this function as a task
            let __inspect_task_id = ::async_inspect::inspector::Inspector::global()
                .register_task(#fn_name_str.to_string());

            ::async_inspect::instrument::set_current_task_id(__inspect_task_id);

            // Execute the original function
            let __inspect_result = async move #instrumented_block;

            let __result = __inspect_result.await;

            // Mark task as completed
            ::async_inspect::inspector::Inspector::global().task_completed(__inspect_task_id);
            ::async_inspect::instrument::clear_current_task_id();

            __result
        }
    };

    output.into()
}

/// Visitor that instruments `.await` expressions
struct AwaitInstrumenter {
    counter: usize,
    fn_name: String,
}

impl VisitMut for AwaitInstrumenter {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // First, recursively visit child expressions BEFORE processing this node
        // This prevents infinite recursion when we replace await expressions
        syn::visit_mut::visit_expr_mut(self, expr);

        // Now check if THIS expression is an await that needs instrumentation
        if let Expr::Await(await_expr) = expr {
            self.counter += 1;
            let label = format!("{}::await#{}", self.fn_name, self.counter);

            // Get the source location
            let location = format!("{}:{}", file!(), line!());

            // Wrap the await with inspection
            // Clone the base to avoid borrow issues
            let base = await_expr.base.clone();

            *expr = syn::parse_quote! {
                {
                    ::async_inspect::instrument::inspect_await_start(#label, Some(#location.to_string()));
                    let __result = #base.await;
                    ::async_inspect::instrument::inspect_await_end(#label);
                    __result
                }
            };
        }
    }
}

/// Attribute macro for inspecting specific code blocks
///
/// # Example
///
/// ```rust,ignore
/// #[async_inspect::inspect]
/// async fn process_data(data: Vec<u8>) -> Result<(), Error> {
///     // This entire block will be tracked
///     let parsed = parse(data)?;
///     let validated = validate(parsed).await?;
///     Ok(validated)
/// }
/// ```
#[proc_macro_attribute]
pub fn inspect(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // For now, just an alias to trace
    trace(_attr, item)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        assert_eq!(2 + 2, 4);
    }
}
