use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    braced,
    ext::IdentExt,
    parenthesized,
    parse::{Parse, ParseStream},
    Ident, Token, Type,
};
use tonic_build::{
    manual::{Method as TonicMethod, Service as TonicService},
    CodeGenBuilder,
};

#[proc_macro]
pub fn rotors(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let descriptor_metadata: Descriptor = syn::parse_macro_input!(item);
    descriptor_metadata.into_token_stream().into()
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(package);
    custom_keyword!(returns);
    custom_keyword!(rpc);
    custom_keyword!(service);
    custom_keyword!(stream);
}

#[derive(Debug)]
struct Descriptor {
    package: Package,
    service: Vec<Service>,
}

impl Parse for Descriptor {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let package = input.parse()?;
        let mut service = vec![];
        while !input.is_empty() {
            let service_metadata = input.parse()?;
            service.push(service_metadata);
        }

        let descriptor = Descriptor { package, service };

        Ok(descriptor)
    }
}

#[derive(Debug)]
struct Package {
    name: String,
}

impl Parse for Package {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::package>()?;
        let mut name = format!("{}", input.parse::<Ident>()?);
        loop {
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![;]) {
                input.parse::<Token![;]>()?;
                break Ok(Package { name });
            } else if lookahead.peek(Token![.]) {
                input.parse::<Token![.]>()?;
                name = format!("{}.{}", name, input.parse::<Ident>()?);
            } else {
                break Err(lookahead.error());
            }
        }
    }
}

#[derive(Debug)]
struct Service {
    name: Ident,
    method: Vec<Method>,
}

impl Parse for Service {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::service>()?;
        let name = input.parse::<Ident>()?;
        let method = {
            let content;
            let _brace_token = braced!(content in input);

            let mut method = vec![];
            while !content.is_empty() {
                let method_descriptor: Method = content.parse()?;
                method.push(method_descriptor)
            }
            method
        };

        Ok(Service { name, method })
    }
}

#[derive(Debug)]
struct Method {
    name: Ident,
    input_type: Type,
    output_type: Type,
    client_streaming: bool,
    server_streaming: bool,
}

impl Parse for Method {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::rpc>()?;
        let name = input.parse::<Ident>()?;

        let (client_streaming, input_type) = parse_method_type(input)?;
        input.parse::<kw::returns>()?;
        let (server_streaming, output_type) = parse_method_type(input)?;

        input.parse::<Token![;]>()?;

        Ok(Method {
            name,
            input_type,
            output_type,
            client_streaming,
            server_streaming,
        })
    }
}

fn parse_method_type(input: ParseStream) -> syn::Result<(bool, Type)> {
    let content;
    parenthesized!(content in input);

    let lookahead = content.lookahead1();
    let streaming = if lookahead.peek(kw::stream) {
        content.parse::<kw::stream>()?;
        true
    } else {
        false
    };

    let r#type = content.parse::<Type>()?;

    Ok((streaming, r#type))
}

impl ToTokens for Descriptor {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let package = &self.package.name;
        let services = self.service.iter().map(|service| {
            let name = service.name.unraw().to_string();

            let mut service_builder = TonicService::builder().name(name).package(package);
            for method in &service.method {
                let route_name = method.name.unraw().to_string();
                let name = route_name.to_snake_case();

                let mut method_builder = TonicMethod::builder()
                    .name(name)
                    .route_name(route_name)
                    .input_type(method.input_type.to_token_stream().to_string())
                    .output_type(method.output_type.to_token_stream().to_string())
                    .codec_path("::rotors::codec::CborCodec");

                if method.client_streaming {
                    method_builder = method_builder.client_streaming();
                }
                if method.server_streaming {
                    method_builder = method_builder.server_streaming();
                }

                service_builder = service_builder.method(method_builder.build());
            }

            service_builder.build()
        });
        let codegen = CodeGenBuilder::new();

        let proto_path = "";
        for service in services {
            let client = codegen.generate_client(&service, proto_path);
            let server = codegen.generate_server(&service, proto_path);
            tokens.extend([client, server]);
        }
    }
}
