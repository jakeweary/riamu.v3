use syn::parse_quote;

pub fn option<T: quote::ToTokens>(x: Option<T>) -> syn::Expr {
  match x {
    Some(x) => parse_quote! { Some(#x) },
    None => parse_quote! { None },
  }
}
