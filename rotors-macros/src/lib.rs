mod client;
mod codegen;
mod descriptor;
mod server;

use self::descriptor::Descriptor;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

#[proc_macro]
pub fn rotors(item: TokenStream) -> TokenStream {
    let descriptor_metadata: Descriptor = parse_macro_input!(item);
    descriptor_metadata.into_token_stream().into()
}
