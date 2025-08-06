// Cargo.toml dependencies needed:
// [dependencies]
// proc-macro2 = "1.0"
// quote = "1.0"
// syn = { version = "2.0", features = ["full"] }

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Adds profiling to a function by inserting a ScopedProfiler at the beginning
///
/// Usage:
/// ```
/// #[profile]
/// pub fn my_function() {
///     // Your code here
/// }
/// ```
///
/// This will transform the function to:
/// ```
/// pub fn my_function() {
///     let _profiler = ScopedProfiler::new("my_function");
///     // Your original code here
/// }
/// ```
#[proc_macro_attribute]
pub fn profile(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;
    let attrs = &input_fn.attrs;

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            let _profiler = client::ScopedProfiler::new(#fn_name_str);
            #block
        }
    };

    TokenStream::from(expanded)
}
