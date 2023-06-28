use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{ext::IdentExt, Ident};

use crate::{
    client::generate_client_mod, descriptor::DescriptorMetadata, server::generate_server_mod,
};

impl ToTokens for DescriptorMetadata {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let package = self.package.as_ref();
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

pub(crate) fn service_fullpath(package: Option<&Ident>, service: &Ident) -> String {
    if let Some(package) = package {
        format!("{}.{}", package.unraw(), service.unraw())
    } else {
        format!("{}", service.unraw())
    }
}
