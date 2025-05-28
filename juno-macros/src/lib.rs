use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, Pat, ReturnType, Type};

#[proc_macro_attribute]
pub fn rpc(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match transform(args.into(), input.into()) {
        Ok(output) => output.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn transform(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let func: ItemFn = syn::parse2(input.clone())?;

    let rpc_type_token = if args.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "RPC type must be specified as 'query' or 'mutation'",
        ));
    } else {
        match args.to_string().trim().to_lowercase().as_str() {
            "query" => quote! { ::juno::router::RpcType::Query },
            "mutation" => quote! { ::juno::router::RpcType::Mutation },
            _ => {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "RPC type must be 'query' or 'mutation'",
                ));
            }
        }
    };

    let og_func_name = &func.sig.ident;
    let func_name_str = og_func_name.to_string();
    let inner_func_name = format_ident!("{}_inner", og_func_name);
    let export_func_name = og_func_name.clone();
    let input_struct_name = format_ident!("{}Input", func_name_str.to_pascal_case());

    let mut input_struct_fields = Vec::new();
    let mut inner_call_args = Vec::new();
    let mut state_arg_actual_type: Option<syn::Type> = None;

    let original_fn_inputs_for_inner_signature = func.sig.inputs.clone();

    for arg in &func.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            let arg_pat = &pat_type.pat;
            let arg_ty = &pat_type.ty;

            if let Type::Path(type_path) = &**arg_ty {
                if let Some(segment) = type_path.path.segments.first() {
                    if segment.ident == "State" {
                        if let syn::PathArguments::AngleBracketed(angle_args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) =
                                angle_args.args.first()
                            {
                                state_arg_actual_type = Some(inner_ty.clone());
                                inner_call_args.push(quote! { state_extractor });
                                continue;
                            }
                        }
                        return Err(syn::Error::new_spanned(
                            arg_ty,
                            "State argument must have a generic type e.g. State<MyState>",
                        ));
                    }
                }
            }

            // Regular argument, add to input struct and params for call
            if let Pat::Ident(pat_ident) = &**arg_pat {
                let ident = &pat_ident.ident;
                input_struct_fields.push(quote! { pub #ident: #arg_ty });
                inner_call_args.push(quote! { deserialized_input.#ident });
            } else {
                return Err(syn::Error::new_spanned(arg_pat, "Unsupported argument pattern in RPC function. Only simple identifiers are supported for non-State arguments."));
            }
        } else {
            return Err(syn::Error::new_spanned(arg, "Unsupported argument type in RPC function. Only typed arguments (e.g., 'name: String') are supported."));
        }
    }

    // Determine the actual T type for Specta and if the original function returns Result<T, RpcError>
    let (output_type_for_specta, original_fn_returns_result): (TokenStream, bool) = {
        match &func.sig.output {
            ReturnType::Default => {
                // Corresponds to `-> ()`. Treat as `T = ()`.
                (quote! { () }, false)
            }
            ReturnType::Type(_, ty_ref) => {
                // Corresponds to `-> SomeType`.
                // We need to check if `SomeType` is `Result<ActualT, RpcError>` or just `ActualT`.
                let ty = &**ty_ref; // Dereference Box<Type> to Type

                if let Type::Path(type_path) = ty {
                    // It's a path type, like `String`, `User`, or `Result<User, RpcError>`.
                    if let Some(last_segment) = type_path.path.segments.last() {
                        if last_segment.ident == "Result" {
                            // It's a `Result<...>`
                            if let syn::PathArguments::AngleBracketed(angle_args) =
                                &last_segment.arguments
                            {
                                if angle_args.args.len() == 2 {
                                    let t_arg = &angle_args.args[0];
                                    let e_arg = &angle_args.args[1];

                                    let actual_t_tokens =
                                        if let syn::GenericArgument::Type(inner_t) = t_arg {
                                            quote! { #inner_t }
                                        } else {
                                            return Err(syn::Error::new_spanned(
                                            t_arg,
                                            "The first type argument of Result (T) must be a type.",
                                        ));
                                        };

                                    // Check if the error part is RpcError
                                    if let syn::GenericArgument::Type(err_ty) = e_arg {
                                        if let Type::Path(err_type_path) = &*err_ty {
                                            if err_type_path
                                                .path
                                                .segments
                                                .last()
                                                .map_or(false, |s| s.ident == "RpcError")
                                            {
                                                // Confirmed: Result<ActualT, RpcError>
                                                (actual_t_tokens, true)
                                            } else {
                                                return Err(syn::Error::new_spanned(
                                                    err_ty,
                                                    "If returning a Result, the error type must be RpcError (e.g., Result<MyType, RpcError>).",
                                                ));
                                            }
                                        } else {
                                            return Err(syn::Error::new_spanned(err_ty, "The second type argument of Result (E) must be a path type (expected RpcError)."));
                                        }
                                    } else {
                                        return Err(syn::Error::new_spanned(e_arg, "The second type argument of Result (E) must be a type."));
                                    }
                                } else {
                                    return Err(syn::Error::new_spanned(
                                        &last_segment.arguments,
                                        "Result must have exactly two type arguments: Result<T, RpcError>.",
                                    ));
                                }
                            } else {
                                return Err(syn::Error::new_spanned(
                                    ty,
                                    "Result type must use angle bracketed arguments like Result<MyType, RpcError>.",
                                ));
                            }
                        } else {
                            // Not `Result<...>`, so it's a direct type `T`.
                            (quote! { #ty }, false)
                        }
                    } else {
                        // Type path has no segments. This implies `ty` itself is the type `T`.
                        (quote! { #ty }, false)
                    }
                } else {
                    // Not a Type::Path (e.g., `(i32, String)`, `[i32; 3]`). This is the type `T`.
                    (quote! { #ty }, false)
                }
            }
        }
    };

    let state_extraction_logic = if let Some(actual_state_type) = &state_arg_actual_type {
        quote! {
            let state_extractor = match axum::extract::State::<#actual_state_type>::from_request_parts(&mut parts, &state_param).await {
                Ok(s) => s,
                Err(rejection) => {
                    // Consider a more specific RpcStatus based on the rejection if possible
                    return ::juno::errors::RpcError::new(
                        ::juno::errors::RpcStatus::InternalServerError,
                        format!("Failed to extract state: {}", rejection),
                    ).into_rpc_response();
                }
            };
        }
    } else {
        quote! {
            // If state_param is not used, and S is (), this is fine.
            // If S is something else, it's passed but ignored by this handler.
            let _ = state_param; // Mark as used to avoid warnings if not used by state_extractor
        }
    };

    let deserialization_logic = if input_struct_fields.is_empty() {
        quote! {
            // Validate that the input is either an empty object, null, or missing entirely
            if let Some(value) = &input_json {
                if !value.is_null() && !value.as_object().map_or(false, |obj| obj.is_empty()) {
                    return ::juno::errors::RpcError::new(
                        ::juno::errors::RpcStatus::BadRequest,
                        "This RPC method does not accept any parameters, but parameters were provided".to_string(),
                    ).into_rpc_response();
                }
            }
        }
    } else {
        quote! {
            let deserialized_input: #input_struct_name = match input_json {
                None => {
                    return ::juno::errors::RpcError::new(
                        ::juno::errors::RpcStatus::BadRequest,
                        "Missing input arguments".to_string(),
                    ).into_rpc_response();
                }
                Some(value) => match serde_json::from_value::<#input_struct_name>(value) {
                    Ok(input) => input,
                    Err(err) => {
                        return ::juno::errors::RpcError::new(
                            ::juno::errors::RpcStatus::BadRequest,
                            format!("Failed to deserialize input for '{}': {}", stringify!(#input_struct_name), err),
                        ).into_rpc_response();
                    }
                },
            };
        }
    };

    let original_func_body = &func.block;
    let original_func_asyncness = &func.sig.asyncness;
    let original_func_output_type = &func.sig.output;
    let original_func_attrs = &func.attrs;

    let (wrapper_fn_generics, state_param_type_for_handler, rpc_method_state_type) =
        if let Some(actual_state_type) = &state_arg_actual_type {
            (
                quote! {},                     // No extra generics for the wrapper fn itself
                quote! { #actual_state_type }, // Handler takes the concrete state type
                quote! { #actual_state_type }, // RpcMethod is for the concrete state type
            )
        } else {
            (
                quote! { <S: Clone + Send + Sync + 'static> }, // Wrapper fn is generic
                quote! { S },                                  // Handler takes generic S
                quote! { S },                                  // RpcMethod is for generic S
            )
        };

    let input_struct_definition = if input_struct_fields.is_empty() {
        quote! {}
    } else {
        quote! {
            #[derive(Debug, serde::Deserialize, specta::Type)]
            struct #input_struct_name {
                #(#input_struct_fields),*
            }
        }
    };

    let input_type_ref_logic = if input_struct_fields.is_empty() {
        quote! { None }
    } else {
        quote! { Some(<#input_struct_name as specta::Type>::reference(types, &[])) }
    };

    let output_type_ref_logic = if output_type_for_specta.to_string() == "()" {
        quote! { None }
    } else {
        quote! { Some(<#output_type_for_specta as specta::Type>::reference(types, &[])) }
    };

    let handler_result_processing = if original_fn_returns_result {
        quote! {
            let result = #inner_func_name(#(#inner_call_args),*).await;
            result.into_rpc_response()
        }
    } else {
        quote! {
            let result_value = #inner_func_name(#(#inner_call_args),*).await;
            use ::juno::response::IntoRpcResponse;
            Ok(result_value).into_rpc_response()
        }
    };

    let gen = quote! {
        #(#original_func_attrs)*
        #original_func_asyncness fn #inner_func_name(#original_fn_inputs_for_inner_signature) #original_func_output_type {
            #original_func_body
        }

        #input_struct_definition

        pub fn #export_func_name #wrapper_fn_generics (
            types: &mut specta::TypeCollection,
        ) -> ::juno::router::RpcMethod<#rpc_method_state_type> {
            let name = #func_name_str;
            let rpc_type = #rpc_type_token;

            let input_type_ref = #input_type_ref_logic;
            let output_type_ref = #output_type_ref_logic;

            let handler = std::sync::Arc::new(
                move |input_json: Option<serde_json::Value>, state_param: #state_param_type_for_handler, mut parts: axum::http::request::Parts| {
                    Box::pin(async move {
                        use axum::extract::FromRequestParts as _;
                        use ::juno::response::IntoRpcResponse as _;

                        #state_extraction_logic
                        #deserialization_logic

                        #handler_result_processing
                    }) as std::pin::Pin<Box<dyn std::future::Future<Output = ::juno::response::RpcResponse> + Send>>
                },
            );

            ::juno::router::RpcMethod {
                name: name.to_string(),
                rpc_type,
                input_type: input_type_ref,
                output_type: output_type_ref,
                handler,
            }
        }
    };

    Ok(gen)
}
