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

#[derive(Default, Debug)]
pub struct DescriptorMetadata {
    pub package: Option<Ident>,
    pub service: Vec<ServiceMetadata>,
}

impl Parse for DescriptorMetadata {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut descriptor_metadata = DescriptorMetadata::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::package) {
                let package = parse_package(input)?;
                if descriptor_metadata.package.is_some() {
                    return Err(syn::Error::new(
                        package.span(),
                        "multiple package definition",
                    ));
                }
                descriptor_metadata.package = Some(package);
            } else if lookahead.peek(kw::service) {
                let service: ServiceMetadata = input.parse()?;
                descriptor_metadata.service.push(service);
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(descriptor_metadata)
    }
}

fn parse_package(input: ParseStream) -> syn::Result<Ident> {
    input.parse::<kw::package>()?;
    let package = input.parse::<Ident>()?;
    input.parse::<Token![;]>()?;

    Ok(package)
}

#[derive(Debug)]
pub struct ServiceMetadata {
    pub name: Ident,
    pub method: Vec<MethodDescriptor>,
}

impl Parse for ServiceMetadata {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::service>()?;
        let name = input.parse::<Ident>()?;
        let method = {
            let content;
            let _brace_token = braced!(content in input);

            let mut method = vec![];
            while !content.is_empty() {
                let method_descriptor: MethodDescriptor = content.parse()?;
                method.push(method_descriptor)
            }
            method
        };

        Ok(ServiceMetadata { name, method })
    }
}

#[derive(Debug)]
pub struct MethodDescriptor {
    pub name: Ident,
    pub input_type: Type,
    pub output_type: Type,
    pub client_streaming: bool,
    pub server_streaming: bool,
}

impl Parse for MethodDescriptor {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::rpc>()?;
        let name = input.parse::<Ident>()?;

        let (client_streaming, input_type) = parse_method_type(input)?;
        input.parse::<kw::returns>()?;
        let (server_streaming, output_type) = parse_method_type(input)?;

        input.parse::<Token![;]>()?;

        Ok(MethodDescriptor {
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
