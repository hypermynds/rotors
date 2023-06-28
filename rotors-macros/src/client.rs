use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ext::IdentExt, Ident};

use crate::{
    codegen::service_fullpath,
    descriptor::{MethodDescriptor, ServiceMetadata},
};

pub fn generate_client_mod(package: Option<&Ident>, descriptor: &ServiceMetadata) -> TokenStream {
    let service_ident = format_service_ident(&descriptor.name);
    let client_mod = format_client_mod(&descriptor.name);

    let connect = generate_connect(&service_ident);
    let constructors = generate_constructors(&service_ident);

    // methods
    let method = descriptor
        .method
        .iter()
        .map(|method| generate_method(package, &descriptor.name, method));

    quote! {
        /// Generated client implementations.
        pub mod #client_mod {
            use super::*;

            use ::rotors::codegen::tonic;
            use tonic::codegen::*;
            use tonic::codegen::http::Uri;

            #[derive(Clone, Debug)]
            pub struct #service_ident<T> {
                inner: tonic::client::Grpc<T>,
            }

            #connect

            impl<T> #service_ident<T>
            where
                T: tonic::client::GrpcService<tonic::body::BoxBody>,
                T::Error: Into<StdError>,
                T::ResponseBody: Body<Data = Bytes> + Send + 'static,
                <T::ResponseBody as Body>::Error: Into<StdError> + Send,
            {
                #constructors
                #(#method)*
            }
        }
    }
}

fn format_service_ident(service_ident: &Ident) -> Ident {
    format_ident!("{}Client", service_ident)
}

fn format_client_mod(service_ident: &Ident) -> Ident {
    let snake_case_service_ident = service_ident.unraw().to_string().to_snake_case();
    format_ident!("{}_client", snake_case_service_ident)
}

fn generate_connect(service_ident: &Ident) -> TokenStream {
    quote! {
        impl #service_ident<tonic::transport::Channel> {
            /// Attempt to create a new client by connecting to a given endpoint.
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }
    }
}

fn generate_constructors(service_ident: &Ident) -> TokenStream {
    quote! {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }

        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }

        pub fn with_interceptor<F>(inner: T, interceptor: F) -> #service_ident<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<<T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody>
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error: Into<StdError> + Send + Sync,
        {
            #service_ident::new(InterceptedService::new(inner, interceptor))
        }

        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }

        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }

        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }

        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
    }
}

fn generate_method(
    package: Option<&Ident>,
    service: &Ident,
    descriptor: &MethodDescriptor,
) -> TokenStream {
    let ident = format_ident!("{}", descriptor.name.unraw().to_string().to_snake_case());

    // request
    let input_type = &descriptor.input_type;
    let input_type = if descriptor.client_streaming {
        quote! {impl tonic::IntoStreamingRequest< Message = #input_type>}
    } else {
        quote! {impl tonic::IntoRequest<#input_type>}
    };

    let req_definition = if descriptor.client_streaming {
        quote! {let mut req = request.into_streaming_request();}
    } else {
        quote! {let mut req = request.into_request();}
    };

    let service_fullpath = service_fullpath(package, service);
    let method_name = descriptor.name.unraw().to_string();

    let req = quote! {
        let codec = ::rotors::codec::CborCodec::default();
        #req_definition
        req.extensions_mut().insert(GrpcMethod::new(#service_fullpath, #method_name));
    };

    // path
    let route = format!("/{}/{}", service_fullpath, method_name);
    let path = quote! {let path = http::uri::PathAndQuery::from_static(#route);};

    // output
    let output_type = &descriptor.output_type;
    let output_type = if descriptor.server_streaming {
        quote! {tonic::codec::Streaming<#output_type>}
    } else {
        quote! {#output_type}
    };

    let res = match (descriptor.client_streaming, descriptor.server_streaming) {
        (true, true) => quote! {self.inner.streaming(req, path, codec)},
        (true, false) => quote! {self.inner.client_streaming(req, path, codec)},
        (false, true) => quote! {self.inner.server_streaming(req, path, codec)},
        (false, false) => quote! {self.inner.unary(req, path, codec)},
    };

    let wait = quote! {
        self.inner.ready().await.map_err(|err| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", err.into())
            )
        })?;
    };

    quote!(
        pub async fn #ident(
            &mut self,
            request: #input_type,
        ) -> tonic::Result<tonic::Response<#output_type>> {
            #wait
            #path
            #req
            #res.await
        }
    )
}
