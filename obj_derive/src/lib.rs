extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{self, parse_macro_input, DeriveInput};
#[proc_macro_attribute]
pub fn mark_obj(attr: TokenStream, item: TokenStream) -> TokenStream {
    
    let attr_ast: syn::Variant = syn::parse(attr).unwrap();
    let item_str = item.to_string();
    let item_input = parse_macro_input!(item as DeriveInput);
    let item_name = item_input.ident.to_string();
    let name = attr_ast.ident.to_string();
    format!(r#"
           {}
           impl IsObj for {} {{
               fn obj_id() -> ObjType {{
                   ObjType::{}
               }}
 
           }}
"#, item_str, item_name, name)
    .parse().unwrap()
}
