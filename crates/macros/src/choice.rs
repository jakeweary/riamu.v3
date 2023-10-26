use darling::export::NestedMeta;
use darling::{FromMeta, Result};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DeriveInput};

#[derive(Debug, FromMeta)]
struct VariantArgs {
  name: Option<String>,
}

pub fn expand(input: TokenStream) -> Result<TokenStream> {
  let input = syn::parse::<DeriveInput>(input)?;

  let Data::Enum(DataEnum { mut variants, .. }) = input.data else {
    unimplemented!()
  };

  let choice_names = variants
    .iter_mut()
    .map(|v| {
      let args = VariantArgs::from_list({
        &std::mem::take(&mut v.attrs)
          .into_iter()
          .map(|attr| NestedMeta::Meta(attr.meta))
          .collect::<Vec<_>>()
      })?;
      Ok(args.name.unwrap_or(v.ident.to_string()))
    })
    .collect::<Result<Vec<_>>>()?;

  let enum_ident = input.ident;
  let variant_idents = variants.iter().map(|v| &v.ident).collect::<Vec<_>>();
  let variant_idents_str = variants.iter().map(|v| v.ident.to_string()).collect::<Vec<_>>();

  let trait_impl = quote! {
    impl<'a> crate::client::CommandOptionTrait<'a> for #enum_ident {
      const TYPE: ::serenity::all::CommandOptionType = ::serenity::all::CommandOptionType::String;
      const CHOICES: Option<&'static [crate::client::CommandOptionChoice]> = Some(&[
        #(crate::client::CommandOptionChoice {
          name: #choice_names,
          value: #variant_idents_str
        }),*
      ]);

      fn extract(value: Option<&::serenity::all::ResolvedOption<'a>>) -> Self {
        match crate::client::CommandOptionTrait::extract(value) {
          #(#variant_idents_str => Self::#variant_idents,)*
          _ => unreachable!()
        }
      }
    }
  };

  Ok(trait_impl.into())
}
