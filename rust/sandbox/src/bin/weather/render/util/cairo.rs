use cairo::*;

pub fn set_font_variations(ctx: &Context, var: &str) -> cairo::Result<()> {
  let opt = FontOptions::new()?;
  opt.set_variations(Some(var));
  ctx.set_font_options(&opt);
  Ok(())
}

pub fn center_text_by_template(ctx: &Context, template: &str, text: &str) -> cairo::Result<()> {
  let pos = ctx.text_extents(template)?.x_advance() / 2.0;
  let neg = ctx.text_extents(text)?.x_advance();
  ctx.rel_move_to(pos - neg, 0.0);
  Ok(())
}
