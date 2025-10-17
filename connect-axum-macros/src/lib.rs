use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemImpl, Path, parse_macro_input};

struct ConnectImplArgs {
    trait_path: Path,
}

impl syn::parse::Parse for ConnectImplArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let trait_path = input.parse()?;
        Ok(ConnectImplArgs { trait_path })
    }
}

/// Macro that implements a Connect service trait and generates axum routes
///
/// Usage: #[connect_impl(path::to::ServiceTrait)]
#[proc_macro_attribute]
pub fn connect_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as ConnectImplArgs);
    let input = parse_macro_input!(input as ItemImpl);

    let trait_path = &args.trait_path;
    let self_ty = &input.self_ty;
    let impl_generics = &input.generics;
    let methods = &input.items;

    // Extract method names and signatures
    let mut method_impls = Vec::new();
    let mut route_handlers = Vec::new();
    let mut route_registrations = Vec::new();

    for item in methods {
        if let syn::ImplItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let method_name_str = method_name.to_string();
            let method_name_pascal = to_pascal_case(&method_name_str);

            // Extract request and response types from the method signature
            // Expected: async fn method_name(&self, request: RequestType) -> Result<ResponseType, ConnectError>
            let inputs = &method.sig.inputs;
            let output = &method.sig.output;

            // Get the request type (second parameter)
            let request_type = if inputs.len() >= 2 {
                if let syn::FnArg::Typed(pat_type) = &inputs[1] {
                    &pat_type.ty
                } else {
                    return syn::Error::new_spanned(inputs, "Expected typed parameter for request")
                        .to_compile_error()
                        .into();
                }
            } else {
                return syn::Error::new_spanned(
                    inputs,
                    "Expected method signature: async fn method(&self, request: RequestType)",
                )
                .to_compile_error()
                .into();
            };

            // Get the response type from Result<ResponseType, _>
            let response_type = match output {
                syn::ReturnType::Type(_, ty) => {
                    // Parse Result<ResponseType, ConnectError>
                    if let syn::Type::Path(type_path) = &**ty {
                        if let Some(segment) = type_path.path.segments.last() {
                            if segment.ident == "Result" {
                                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                                {
                                    if let Some(syn::GenericArgument::Type(resp_ty)) =
                                        args.args.first()
                                    {
                                        resp_ty
                                    } else {
                                        return syn::Error::new_spanned(
                                            output,
                                            "Expected Result<ResponseType, ConnectError>",
                                        )
                                        .to_compile_error()
                                        .into();
                                    }
                                } else {
                                    return syn::Error::new_spanned(
                                        output,
                                        "Expected Result<ResponseType, ConnectError>",
                                    )
                                    .to_compile_error()
                                    .into();
                                }
                            } else {
                                return syn::Error::new_spanned(
                                    output,
                                    "Expected Result return type",
                                )
                                .to_compile_error()
                                .into();
                            }
                        } else {
                            return syn::Error::new_spanned(output, "Expected Result return type")
                                .to_compile_error()
                                .into();
                        }
                    } else {
                        return syn::Error::new_spanned(output, "Expected Result return type")
                            .to_compile_error()
                            .into();
                    }
                }
                _ => {
                    return syn::Error::new_spanned(output, "Expected Result return type")
                        .to_compile_error()
                        .into();
                }
            };

            let method_block = &method.block;

            // Generate the trait implementation for this method
            method_impls.push(quote! {
                async fn #method_name(&self, request: #request_type)
                    -> Result<#response_type, connect_axum::ConnectError>
                #method_block
            });

            // Generate the route handler
            let handler_name = syn::Ident::new(
                &format!("__connect_handler_{}", method_name),
                method_name.span(),
            );

            route_handlers.push(quote! {
                async fn #handler_name(
                    axum::extract::State(service): axum::extract::State<std::sync::Arc<#self_ty>>,
                    req: axum::extract::Request,
                ) -> Result<axum::response::Response, connect_axum::ConnectError> {
                    use connect_axum::ConnectMessage;

                    // Parse the Connect request
                    let connect_req = connect_axum::extract_connect_request(req).await?;

                    // Decode the request message
                    let request_msg = match connect_req.encoding {
                        connect_axum::Encoding::Json => {
                            <#request_type as ConnectMessage>::decode_json(&connect_req.message)?
                        }
                        connect_axum::Encoding::Proto => {
                            <#request_type as ConnectMessage>::decode_proto(&connect_req.message)?
                        }
                    };

                    // Call the service method
                    let response_msg = service.#method_name(request_msg).await?;

                    // Encode the response
                    let response_bytes = match connect_req.encoding {
                        connect_axum::Encoding::Json => {
                            <#response_type as ConnectMessage>::encode_json(&response_msg)?
                        }
                        connect_axum::Encoding::Proto => {
                            <#response_type as ConnectMessage>::encode_proto(&response_msg)?
                        }
                    };

                    // Build the HTTP response
                    connect_axum::build_connect_response(response_bytes, connect_req.encoding)
                }
            });

            // Generate route registration
            // Get the service name from the trait path (last segment)
            let service_name = if let Some(last) = trait_path.segments.last() {
                last.ident.to_string()
            } else {
                "UnknownService".to_string()
            };

            // Construct the full path
            // TODO: Get package from metadata instead of hardcoding
            let route_path = format!("/greet.v1.{}/{}", service_name, method_name_pascal);

            route_registrations.push(quote! {
                .route(#route_path, post(#handler_name).get(#handler_name))
            });
        }
    }

    // Generate the output
    let expanded = quote! {
        // Original impl block for the trait
        impl #impl_generics #trait_path for #self_ty {
            #(#method_impls)*
        }

        // Additional impl block with router generation
        impl #impl_generics #self_ty {
            pub fn into_router(self) -> axum::Router {
                use axum::routing::{post, get};

                let service = std::sync::Arc::new(self);

                // Generate the handler functions in scope
                #(#route_handlers)*

                // Build router with all routes
                axum::Router::new()
                    #(#route_registrations)*
                    .with_state(service)
            }
        }
    };

    TokenStream::from(expanded)
}

/// Convert snake_case to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}
