use std::borrow::Cow;

use lib::text::style;
use serenity::all::*;

use crate::client::{Context, Result};

#[derive(macros::Choice)]
enum Style {
  #[name = "Regional indicators"]
  RegionalIndicators,
  #[name = "Full width"]
  FullWidth,
  #[name = "Monospace"]
  Monospace,
  #[name = "Double struck"]
  DoubleStruck,
  #[name = "Fractur"]
  Fractur,
  #[name = "Fractur (bold)"]
  FracturBold,
  #[name = "Script"]
  Script,
  #[name = "Script (bold)"]
  ScriptBold,
  #[name = "Serif (bold)"]
  SerifBold,
  #[name = "Serif (italic)"]
  SerifItalic,
  #[name = "Serif (bold italic)"]
  SerifBoldItalic,
  #[name = "Sans-Serif"]
  SansSerif,
  #[name = "Sans-Serif (bold)"]
  SansSerifBold,
  #[name = "Sans-Serif (italic)"]
  SansSerifItalic,
  #[name = "Sans-Serif (bold italic)"]
  SansSerifBoldItalic,
}

#[macros::command(description = "Text style")]
pub async fn run(ctx: &Context<'_>, input: &str, style: Style) -> Result<()> {
  let f = match style {
    Style::RegionalIndicators => style::regional_indicators,
    Style::FullWidth => style::full_width,
    Style::Monospace => style::monospace,
    Style::DoubleStruck => style::double_struck,
    Style::Fractur => style::fractur::regular,
    Style::FracturBold => style::fractur::bold,
    Style::Script => style::script::regular,
    Style::ScriptBold => style::script::bold,
    Style::SerifBold => style::serif::bold,
    Style::SerifItalic => style::serif::italic,
    Style::SerifBoldItalic => style::serif::bold_italic,
    Style::SansSerif => style::sans_serif::regular,
    Style::SansSerifBold => style::sans_serif::bold,
    Style::SansSerifItalic => style::sans_serif::italic,
    Style::SansSerifBoldItalic => style::sans_serif::bold_italic,
  };

  let input = match style {
    Style::RegionalIndicators => Cow::Owned(format!("`{}`", input)),
    _ => Cow::Borrowed(input),
  };

  let output = input.chars().map(f).collect::<String>();

  tracing::debug!("sending response…");
  let msg = CreateInteractionResponseMessage::new().content(output);
  let msg = CreateInteractionResponse::Message(msg);
  ctx.event.create_response(ctx, msg).await?;

  Ok(())
}
