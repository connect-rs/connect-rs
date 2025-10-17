use heck::{ToPascalCase, ToSnakeCase};
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{ItemImpl, Path, parse_macro_input, spanned::Spanned};

/// Example: #[connect_rs_impl(v1::auth::AuthService)]
#[proc_macro_error]
#[proc_macro_attribute]
pub fn connect_rs_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let trait_path = parse_macro_input!(args as Path);
    let input = parse_macro_input!(input as ItemImpl);

    let self_ty = &input.self_ty; // Self type
    let impl_generics = &input.generics; // Generics, like <A + B + C>
    let items = &input.items; // Things like fn, const, type, macro

    // Extract the service name from the trait path
    let service_name = if let Some(last) = trait_path.segments.last() {
        last.ident.to_string()
    } else {
        abort!(
            trait_path,
            "Invalid service name";
            help = "Expected format: package::v1::ServiceName"
        );
    };

    // Build the path to the metadata module
    // For trait path like `auth::v1::AuthService`, metadata is at `auth::v1::__auth_service_meta`
    let meta_module_name = format!("__{}_meta", service_name.to_snake_case());
    let meta_module_ident = syn::Ident::new(&meta_module_name, trait_path.span());

    // Build the full path to the metadata module by taking all segments except the last (service name)
    // and appending the metadata module name
    let parent_segments: Vec<_> = trait_path
        .segments
        .iter()
        .take(trait_path.segments.len().saturating_sub(1))
        .collect();

    let meta_path = if parent_segments.is_empty() {
        // If no parent modules, just use the meta module name
        quote! { #meta_module_ident }
    } else {
        // Build path like auth::v1::__auth_service_meta
        quote! { #(#parent_segments)::* :: #meta_module_ident }
    };

    // Extract method names and signatures
    let mut method_impls = Vec::new();
    let mut route_handlers = Vec::new();
    let mut route_registrations = Vec::new();

    for item in items {
        if let syn::ImplItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let method_name_str = method_name.to_string();

            // Convert snake_case to PascalCase then uppercase to match protoc constant
            // get_user -> GetUser -> GETUSER
            let pascal_case = method_name_str.to_pascal_case();
            let method_name_upper = pascal_case.to_uppercase();
            let method_const_ident = syn::Ident::new(&method_name_upper, method_name.span());

            // Extract request and response types from the method signature
            // Expected: async fn method_name(&self, request: RequestType) -> Result<ResponseType, ConnectError>
            let inputs = &method.sig.inputs;
            let output = &method.sig.output;

            // Get the request type (second parameter)
            let request_type = if inputs.len() >= 2 {
                if let syn::FnArg::Typed(pat_type) = &inputs[1] {
                    &pat_type.ty
                } else {
                    abort!(inputs, "Expected typed parameter for request");
                }
            } else {
                abort!(
                    inputs,
                    "Expected method signature: async fn method(&self, request: RequestType)"
                );
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
                                        abort!(
                                            output,
                                            "Expected Result<ResponseType, ConnectError>"
                                        );
                                    }
                                } else {
                                    abort!(output, "Expected Result<ResponseType, ConnectError>");
                                }
                            } else {
                                abort!(output, "Expected Result return type");
                            }
                        } else {
                            abort!(output, "Expected Result return type");
                        }
                    } else {
                        abort!(output, "Expected Result return type");
                    }
                }
                _ => {
                    abort!(output, "Expected Result return type");
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
                &format!("__connect_handler_{method_name}"),
                method_name.span(),
            );

            route_handlers.push(quote! {
                async fn #handler_name(
                    axum::extract::State(service): axum::extract::State<std::sync::Arc<#self_ty>>,
                    req: axum::extract::Request,
                ) -> Result<axum::response::Response, connect_axum::ConnectError> {
                    use connect_axum::{ConnectMessageJson, ConnectMessageProto};

                    // Parse the incoming Connect request
                    let connect_req = connect_axum::parse_connect_request(req).await?;

                    // Decode the request message
                    let request_msg = match connect_req.encoding {
                        connect_axum::Encoding::Json => {
                            <#request_type as ConnectMessageJson>::decode_json(&connect_req.message)?
                        }
                        connect_axum::Encoding::Proto => {
                            <#request_type as ConnectMessageProto>::decode_proto(&connect_req.message)?
                        }
                    };

                    // Call the service method
                    let response_msg = service.#method_name(request_msg).await?;

                    // Encode the response
                    let response_bytes = match connect_req.encoding {
                        connect_axum::Encoding::Json => {
                            <#response_type as ConnectMessageJson>::encode_json(&response_msg)?
                        }
                        connect_axum::Encoding::Proto => {
                            <#response_type as ConnectMessageProto>::encode_proto(&response_msg)?
                        }
                    };

                    // The final HTTP response
                    connect_axum::encode_http_response(response_bytes, connect_req.encoding)
                }
            });

            // For idempotent methods, use GET only
            // For non-idempotent methods, use both POST and GET
            route_registrations.push(quote! {
                .route(
                    #meta_path::#method_const_ident.path,
                    if #meta_path::#method_const_ident.idempotent {
                        axum::routing::method_routing::MethodRouter::new()
                            .get(#handler_name)
                            .post(#handler_name)
                    } else {
                        axum::routing::post(#handler_name)
                    }
                )
            });
        }
    }

    let expanded = quote! {
        impl #impl_generics #trait_path for #self_ty {
            #(#method_impls)*
        }

        impl #impl_generics #self_ty {
            pub fn into_router(self) -> axum::Router {
                use axum::routing::{post, get};

                let service = std::sync::Arc::new(self);

                #(#route_handlers)*

                axum::Router::new()
                    #(#route_registrations)*
                    .with_state(service)
            }
        }
    };

    TokenStream::from(expanded)
}
