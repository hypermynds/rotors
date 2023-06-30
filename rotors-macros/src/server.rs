use heck::{ToPascalCase, ToSnakeCase, ToUpperCamelCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ext::IdentExt, Ident};

use crate::{
    codegen::service_fullpath,
    descriptor::{Method, Service},
};

pub fn generate_server_mod(package: &Ident, descriptor: &Service) -> TokenStream {
    let service_ident = format_service_ident(&descriptor.name);
    let trait_ident = format_trait_ident(&descriptor.name);
    let client_mod = format_client_mod(&descriptor.name);

    let service_name = format!("{}.{}", package.unraw(), descriptor.name.unraw());

    let trait_method = descriptor.method.iter().map(generate_trait_method);

    let clone = generate_clone(&service_ident, &trait_ident);
    let constructors = generate_constructors();
    let service = generate_service(&service_ident, &trait_ident, package, descriptor);
    let named_service = generate_named_service(&service_ident, &trait_ident, &service_name);

    quote! {
        /// Generated server implementations.
        pub mod #client_mod {
            use super::*;
            use ::rotors::codegen::tonic;
            use tonic::codegen::*;

            #[async_trait]
            pub trait #trait_ident: Send + Sync + 'static {
                #(#trait_method)*
            }

            #[derive(Debug)]
            pub struct #service_ident<T: #trait_ident> {
                inner: std::sync::Arc<T>,
                accept_compression_encodings: EnabledCompressionEncodings,
                send_compression_encodings: EnabledCompressionEncodings,
                max_decoding_message_size: Option<usize>,
                max_encoding_message_size: Option<usize>,
            }

            #clone

            impl<T: #trait_ident> #service_ident<T> {
                #constructors
            }

            #service
            #named_service
        }
    }
}

fn format_service_ident(service_ident: &Ident) -> Ident {
    format_ident!(
        "{}Server",
        service_ident.unraw().to_string().to_pascal_case()
    )
}

fn format_trait_ident(service_ident: &Ident) -> Ident {
    format_ident!("{}", service_ident.unraw().to_string().to_pascal_case())
}

fn format_client_mod(service_ident: &Ident) -> Ident {
    let snake_case_service_ident = service_ident.unraw().to_string().to_snake_case();
    format_ident!("{}_server", snake_case_service_ident)
}

fn generate_trait_method(method: &Method) -> TokenStream {
    let ident = format_ident!("{}", method.name.unraw().to_string().to_snake_case());

    // input
    let input_type = &method.input_type;
    let input_type = if method.client_streaming {
        quote! {tonic::Streaming<#input_type>}
    } else {
        quote! {#input_type}
    };

    // output
    let output_type = &method.output_type;

    if method.server_streaming {
        let stream = format_ident!(
            "{}Stream",
            method.name.unraw().to_string().to_upper_camel_case()
        );

        quote! {
            type #stream: futures_core::Stream<Item = #output_type>;

            async fn #ident(
                &self,
                request: tonic::Request<#input_type>,
            ) -> tonic::Result<tonic::Response<Self::#stream>>;
        }
    } else {
        quote! {
            async fn #ident(
                &self,
                request: tonic::Request<#input_type>,
            ) -> tonic::Result<tonic::Response<#output_type>>;
        }
    }
}

fn generate_clone(ident: &Ident, trait_ident: &Ident) -> TokenStream {
    quote! {
        impl<T: #trait_ident> Clone for #ident<T> {
            fn clone(&self) -> Self {
                Self {
                    inner: self.inner.clone(),
                    accept_compression_encodings: self.accept_compression_encodings,
                    send_compression_encodings: self.send_compression_encodings,
                    max_decoding_message_size: self.max_decoding_message_size,
                    max_encoding_message_size: self.max_encoding_message_size,
                }
            }
        }
    }
}

fn generate_constructors() -> TokenStream {
    quote! {
        pub fn new(inner: T) -> Self {
            Self::from_arc(std::sync::Arc::new(inner))
        }

        pub fn from_arc(inner: std::sync::Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }

        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }

        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }

        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }

        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }

        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
}

fn generate_service(
    service_ident: &Ident,
    trait_ident: &Ident,
    package: &Ident,
    descriptor: &Service,
) -> TokenStream {
    let default_route = quote! {
        _ => Box::pin(async move {
            Ok(http::Response::builder()
                .status(200)
                .header("grpc-status", i32::from(tonic::Code::Unimplemented))
                .header("content-type", "application/grpc")
                .body(empty_body())
                .unwrap())
        })
    };

    let server_routes = descriptor
        .method
        .iter()
        .map(|method| generate_server_route(trait_ident, package, &descriptor.name, method));

    quote! {
        impl<T, B> tonic::codegen::Service<http::Request<B>> for #service_ident<T>
        where
            T: #trait_ident,
            B: Body + Send + 'static,
            B::Error: Into<StdError> + Send + 'static,
        {
            type Response = http::Response<tonic::body::BoxBody>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;

            fn poll_ready(
                &mut self,
                _cx: &mut Context<'_>
            ) -> Poll<std::result::Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }

            fn call(
                &mut self,
                req: http::Request<B>
            ) -> Self::Future {
                let inner = self.inner.clone();
                match req.uri().path() {
                    #(#server_routes),*
                    #default_route
                }
            }
        }
    }
}

fn generate_server_route(
    trait_ident: &Ident,
    package: &Ident,
    service: &Ident,
    descriptor: &Method,
) -> TokenStream {
    let service_fullpath = service_fullpath(package, service);
    let method_name = descriptor.name.unraw().to_string();
    let route = format!("/{}/{}", service_fullpath, method_name);

    let grpc = quote! {
        let method = Svc(inner);
        let codec = ::rotors::codec::CborCodec::default();
        let mut grpc = tonic::server::Grpc::new(codec)
            .apply_compression_config(
                self.accept_compression_encodings,
                self.send_compression_encodings,
            )
            .apply_max_message_size_config(
                self.max_decoding_message_size,
                self.max_encoding_message_size,
            );
    };

    let input_type = &descriptor.input_type;
    let output_type = &descriptor.output_type;
    let output_stream = format_ident!(
        "{}Stream",
        descriptor.name.unraw().to_string().to_upper_camel_case()
    );

    let trait_method_ident =
        format_ident!("{}", descriptor.name.unraw().to_string().to_snake_case());

    let fut = match (descriptor.client_streaming, descriptor.server_streaming) {
        (false, false) => quote! {
            struct Svc<T>(std::sync::Arc<T>);

            impl<T: #trait_ident> tonic::server::UnaryService<#input_type> for Svc<T> {
                type Response = #output_type;
                type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;

                fn call(
                    &mut self,
                    request: tonic::Request<#input_type>,
                ) -> Self::Future {
                    let inner = self.0.clone();
                    Box::pin(async move { inner.#trait_method_ident(request).await })
                }
            }

            #grpc
            let fut = async move { Ok(grpc.unary(method, req).await) };
        },
        (false, true) => quote! {
            struct Svc<T>(std::sync::Arc<T>);

            impl<T: #trait_ident> tonic::server::ServerStreamingService<#input_type> for Svc<T> {
                type Response = #output_type;
                type ResponseStream = T::#output_stream;
                type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;

                fn call(
                    &mut self,
                    request: tonic::Request<#input_type>,
                ) -> Self::Future {
                    let inner = self.0.clone();
                    Box::pin(async move { inner.#trait_method_ident(request).await })
                }
            }

            #grpc
            let fut = async move { Ok(grpc.server_streaming(method, req).await) };
        },
        (true, false) => quote! {
            struct Svc<T>(std::sync::Arc<T>);

            impl<T: #trait_ident> tonic::server::ClientStreamingService<#input_type> for Svc<T> {
                type Response = #output_type;
                type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;

                fn call(
                    &mut self,
                    request: tonic::Request<tonic::Streaming<#input_type>>,
                ) -> Self::Future {
                    let inner = self.0.clone();
                    Box::pin(async move { inner.#trait_method_ident(request).await })
                }
            }

            #grpc
            let fut = async move { Ok(grpc.client_streaming(method, req).await) };
        },
        (true, true) => quote! {
            struct Svc<T>(std::sync::Arc<T>);

            impl<T: #trait_ident> tonic::server::StreamingService<#input_type> for Svc<T> {
                type Response = #output_type;
                type ResponseStream = T::#output_stream;
                type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;

                fn call(
                    &mut self,
                    request: tonic::Request<tonic::Streaming<#input_type>>,
                ) -> Self::Future {
                    let inner = self.0.clone();
                    Box::pin(async move { inner.#trait_method_ident(request).await })
                }
            }

            #grpc
            let fut = async move { Ok(grpc.streaming(method, req).await) };
        },
    };

    quote!(
        #route => {
            #fut
            Box::pin(fut)
        }
    )
}

fn generate_named_service(
    service_ident: &Ident,
    trait_ident: &Ident,
    service_name: &str,
) -> TokenStream {
    quote! {
        impl<T: #trait_ident> tonic::server::NamedService for #service_ident<T> {
            const NAME: &'static str = #service_name;
        }
    }
}
