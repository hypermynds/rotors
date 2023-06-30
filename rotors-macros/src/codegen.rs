use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{ext::IdentExt, Ident};

use crate::{client::generate_client_mod, descriptor::Descriptor, server::generate_server_mod};

impl ToTokens for Descriptor {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let package = &self.package.name;
        let client = self
            .service
            .iter()
            .map(|descriptor| generate_client_mod(package, descriptor));
        let server = self
            .service
            .iter()
            .map(|descriptor| generate_server_mod(package, descriptor));

        tokens.extend(client.chain(server))
    }
}

pub(crate) fn service_fullpath(package: &Ident, service: &Ident) -> String {
    format!("{}.{}", package.unraw(), service.unraw())
}
