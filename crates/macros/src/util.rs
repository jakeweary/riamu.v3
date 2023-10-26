pub fn option<T: quote::ToTokens>(x: Option<T>) -> syn::Expr {
  match x {
    Some(x) => syn::parse_quote! { Some(#x) },
    None => syn::parse_quote! { None },
  }
}
