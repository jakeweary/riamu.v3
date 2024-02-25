use lib::api::_4chan::parse_url;
use lib::api::_4chan::{catalog::Catalog, thread::Thread};
use lib::fmt::plural::Plural;
use lib::{fmt, html};
use serenity::all::*;

use crate::client::{err, Context, Result};

#[macros::command(desc = "Repost something from 4chan")]
pub async fn repost(ctx: &Context<'_>, #[desc = "4chan thread url"] url: &str) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::debug!("parsing url…");
  let Some((domain, board_id, thread_id, post_id)) = parse_url(url) else {
    err::message!("failed to parse url");
  };

  tracing::debug!("getting thread…");
  let thread = Thread::get(board_id, thread_id).await?;

  let post_id = post_id.unwrap_or(thread_id);
  reply(ctx, domain, board_id, post_id, &thread).await
}

#[macros::command(desc = "Random 4chan post")]
pub async fn random_post(ctx: &Context<'_>, #[desc = "4chan board id"] board: &str) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::debug!("getting catalog…");
  let catalog = Catalog::get(board).await?;
  let (thread, post_index) = catalog.random(|t| 1 + t.replies as usize);

  tracing::debug!("getting thread…");
  let thread = Thread::get(board, thread.id).await?;
  let post = &thread.posts[post_index];

  reply(ctx, "boards.4chan.org", board, post.id, &thread).await
}

#[macros::command(desc = "Random 4chan post with a file attachment")]
pub async fn random_file(ctx: &Context<'_>, #[desc = "4chan board id"] board: &str) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::debug!("getting catalog…");
  let catalog = Catalog::get(board).await?;
  let (thread, file_index) = catalog.random(|t| 1 + t.images as usize);

  tracing::debug!("getting thread…");
  let thread = Thread::get(board, thread.id).await?;
  let posts = thread.posts.iter();
  let post = posts.filter(|p| p.file.is_some()).nth(file_index).unwrap();

  reply(ctx, "boards.4chan.org", board, post.id, &thread).await
}

// ---

async fn reply(ctx: &Context<'_>, domain: &str, board_id: &str, post_id: u64, thread: &Thread) -> Result<()> {
  let (post, post_index) = thread.get_post_by_id(post_id).unwrap();
  let op = &thread.posts[0];

  let url = format!("https://{}/{}/thread/{}#p{}", domain, board_id, op.id, post.id);

  let title = Option::or(op.subject.as_deref(), op.comment.as_deref());
  let title = title.map(html::strip);
  let title = title.as_deref().unwrap_or("<no subject>");
  let title = fmt::ellipsis(&title, 100);
  let title = fmt::line_ellipsis(&title, 2);

  let content = post.render(&url.parse()?);
  let content = fmt::ellipsis(&content, 4096);

  let footer = format! { "/{}/{} · #{}/{} · {}",
    board_id, op.id, post_index + 1, thread.posts.len(),
    thread.find_replies_to(post.id).count().plural("reply", "replies")
  };

  let embed = CreateEmbed::new()
    .timestamp(Timestamp::from_unix_timestamp(post.timestamp as i64)?)
    .description(content)
    .author(CreateEmbedAuthor::new(title).url(url))
    .footer(CreateEmbedFooter::new(footer));

  let files = match &post.file {
    Some(file) => {
      tracing::debug!("attaching files…");
      let url = format!("https://i.4cdn.org/{}/{}{}", board_id, file.id, file.ext);
      let att = CreateAttachment::url(ctx, &url).await?;
      EditAttachments::new().add(att)
    }
    None => EditAttachments::new(),
  };

  tracing::debug!("sending response…");
  let edit = EditInteractionResponse::new().attachments(files);
  ctx.event.edit_response(ctx, edit.embed(embed)).await?;

  Ok(())
}
