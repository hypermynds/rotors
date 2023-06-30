use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    Ident, Token, Type,
};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(package);
    custom_keyword!(returns);
    custom_keyword!(rpc);
    custom_keyword!(service);
    custom_keyword!(stream);
}

#[derive(Debug)]
pub struct Descriptor {
    pub package: Package,
    pub service: Vec<Service>,
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
pub struct Package {
    pub name: Ident,
}

impl Parse for Package {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::package>()?;
        let name = input.parse::<Ident>()?;
        input.parse::<Token![;]>()?;

        Ok(Package { name })
    }
}

#[derive(Debug)]
pub struct Service {
    pub name: Ident,
    pub method: Vec<Method>,
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
pub struct Method {
    pub name: Ident,
    pub input_type: Type,
    pub output_type: Type,
    pub client_streaming: bool,
    pub server_streaming: bool,
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
