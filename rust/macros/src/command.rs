use darling::export::NestedMeta;
use darling::{FromMeta, Result};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, Pat};

use crate::util;

#[derive(Debug, FromMeta)]
struct FnArgs {
  #[darling(default)]
  owner_only: bool,
  desc: Option<String>,
}

#[derive(Debug, FromMeta)]
struct FnInputArgs {
  name: Option<String>,
  desc: Option<String>,
}

pub fn expand(args: TokenStream, input: TokenStream) -> Result<TokenStream> {
  let mut function = syn::parse::<ItemFn>(input)?;

  let meta_list = NestedMeta::parse_meta_list(args.into())?;
  let fn_args = FnArgs::from_list(&meta_list)?;

  let cmd_desc = util::option(fn_args.desc);
  let cmd_options = command_options(&mut function)?;
  let cmd_owner_only = fn_args.owner_only;

  let fn_vis = &function.vis;
  let fn_async = &function.sig.asyncness;
  let fn_ident = &function.sig.ident;
  let fn_inputs = &function.sig.inputs;
  let fn_output = &function.sig.output;
  let fn_block = &function.block;

  let fn_inner_ident = format_ident!("{}_inner", fn_ident);

  let (cmd_context, cmd_option_names) = {
    let mut cmd_option_names = fn_inputs
      .into_iter()
      .map(|input| match input {
        FnArg::Typed(input) => &*input.pat,
        _ => unimplemented!(),
      })
      .collect::<Vec<_>>();

    let cmd_context = cmd_option_names.remove(0);
    (cmd_context, cmd_option_names)
  };

  let cmd_option_names_static = cmd_option_names
    .iter()
    .map(|name| match &**name {
      Pat::Ident(ident) => {
        let span = ident.ident.span();
        let ident = ident.ident.to_string().to_uppercase();
        format_ident!("{}", ident, span = span)
      }
      _ => unimplemented!(),
    })
    .collect::<Vec<_>>();

  let command = quote! {
    #fn_async fn #fn_inner_ident(#fn_inputs) #fn_output #fn_block

    #fn_vis fn #fn_ident(name: &'static str) -> crate::client::Command {
      use crate::client::*;
      use std::{future::Future, pin::Pin};

      #(static #cmd_option_names_static: CommandOption = #cmd_options;)*

      fn run<'a>(#cmd_context: &'a Context<'_>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async {
          #(let #cmd_option_names = CommandOptionTrait::extract({
            #cmd_context.options.iter().find(|o| o.name == #cmd_option_names_static.name)
          });)*
          #fn_inner_ident(#cmd_context, #(#cmd_option_names),*).await
        })
      }

      Command {
        name,
        description: #cmd_desc,
        owner_only: #cmd_owner_only,

        run,
        options: [#(&#cmd_option_names_static),*].into(),
      }
    }
  };

  Ok(command.into())
}

fn command_options(function: &mut ItemFn) -> Result<Vec<proc_macro2::TokenStream>> {
  let inputs = function.sig.inputs.iter_mut().skip(1);

  inputs
    .map(|fn_input| {
      let fn_input = match fn_input {
        FnArg::Typed(input) => input,
        _ => unimplemented!(),
      };
      let fn_input_args = FnInputArgs::from_list({
        &std::mem::take(&mut fn_input.attrs)
          .into_iter()
          .map(|attr| NestedMeta::Meta(attr.meta))
          .collect::<Vec<_>>()
      })?;

      let name = fn_input_args.name.unwrap_or_else(|| match &*fn_input.pat {
        Pat::Ident(ident) => ident.ident.to_string(),
        _ => unimplemented!(),
      });
      let desc = util::option(fn_input_args.desc);
      let ty = &*fn_input.ty;

      let option = quote! {
        CommandOption {
          name: #name,
          description: #desc,
          choices: <#ty as CommandOptionTrait>::CHOICES,
          required: <#ty as CommandOptionTrait>::REQUIRED,
          ty: <#ty as CommandOptionTrait>::TYPE,
        }
      };

      Ok(option)
    })
    .collect()
}
