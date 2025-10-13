use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(ConfigParse)]
pub fn derive_config_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("ConfigParse only supports structs with named fields"),
        },
        _ => panic!("ConfigParse only supports structs"),
    };

    let parse_arms = fields.iter().map(|f| {
        let field_name = &f.ident;
        let field_name_str = field_name.as_ref().unwrap().to_string();
        let field_type = &f.ty;

        // Check if it's a bool type
        let is_bool = quote!(#field_type).to_string().contains("bool");

        if is_bool {
            quote! {
                #field_name_str => {
                    config.#field_name = match value.to_lowercase().as_str() {
                        "true" | "1" | "yes" | "on" => true,
                        "false" | "0" | "no" | "off" => false,
                        _ => config.#field_name,
                    };
                }
            }
        } else {
            quote! {
                #field_name_str => {
                    if let Ok(v) = value.parse() {
                        config.#field_name = v;
                    }
                }
            }
        }
    });

    let expanded = quote! {
        impl #name {
            /// Load config from a file, falling back to defaults for missing values
            pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
                let contents = std::fs::read_to_string(path)?;
                Ok(Self::parse(&contents))
            }

            /// Parse config from a string in key=value format
            fn parse(contents: &str) -> Self {
                let mut config = Self::default();

                for line in contents.lines() {
                    let line = line.trim();
                    
                    // Skip empty lines and comments
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }

                    // Split on first '=' only
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim();
                        let value = value.trim();

                        match key {
                            #(#parse_arms)*
                            _ => {} // Ignore unknown keys
                        }
                    }
                }

                config
            }

            /// Load config from file if it exists, otherwise return defaults
            pub fn load_or_default<P: AsRef<std::path::Path>>(path: P) -> Self {
                Self::from_file(path).unwrap_or_default()
            }
        }
    };

    TokenStream::from(expanded)
}
