use proc_macro::TokenStream;

mod choice;
mod command;
mod util;

/// # Examples
///
/// ```ignore
/// use crate::client::{command, Context, Result};
///
/// #[command(description = "Performs some math operation")]
/// async fn math_op(
///   ctx: &Context<'_>,
///   #[name = "1st"]
///   #[description = "1st operand"]
///   x: f64,
///   #[name = "2nd"]
///   #[description = "2nd operand"]
///   y: f64,
/// ) -> Result<()> {
///   todo!()
/// }
/// ```
#[proc_macro_attribute]
pub fn command(args: TokenStream, input: TokenStream) -> TokenStream {
  command::expand(args, input).unwrap_or_else(|e| e.write_errors().into())
}

/// # Examples
///
/// ```ignore
/// use crate::client::{command, Context, Result};
///
/// #[derive(macros::Choice)]
/// enum Op {
///   #[name = "addition"]
///   Add,
///   #[name = "subtraction"]
///   Sub,
///   #[name = "multiplication"]
///   Mul,
///   #[name = "division"]
///   Div,
/// }
///
/// #[command]
/// async fn math_op(ctx: &Context<'_>, op: Op, x: f64, y: f64) -> Result<()> {
///   todo!()
/// }
/// ```
#[proc_macro_derive(Choice, attributes(name))]
pub fn choice(input: TokenStream) -> TokenStream {
  choice::expand(input).unwrap_or_else(|e| e.write_errors().into())
}
