use std::iter;

use api::_2ch::{catalog, parse_url};
use api::_2ch::{catalog::Catalog, thread::Thread};
use fmt::plural::Plural;
use serenity::all::*;
use util::html;

use crate::client::{err, Context, Result};

#[macros::command(desc = "Repost something from 2ch")]
pub async fn repost(ctx: &Context<'_>, url: &str) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::debug!("parsing url…");
  let Some((domain, board_id, thread_id, post_id)) = parse_url(url) else {
    err::message!("failed to parse url");
  };

  tracing::debug!("getting thread…");
  let thread = Thread::get(domain, board_id, thread_id).await?;

  let post_id = post_id.unwrap_or(thread_id);
  reply(ctx, domain, board_id, post_id, &thread).await
}

#[macros::command(desc = "Random 2ch post")]
pub async fn random(
  ctx: &Context<'_>,
  #[desc = "2ch board id"] board: &str,
  #[desc = "Only threads where subject matches this regex"] include: Option<&str>,
  #[desc = "Only threads where subject doesn't match this regex"] exclude: Option<&str>,
) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::debug!("getting catalog…");
  let catalog = Catalog::get(board).await?;
  let filter = catalog::thread_filter(include, exclude)?;
  let Some((thread, file_index)) = catalog.random(filter, |t| t.files_count as usize) else {
    err::message!("no results");
  };

  tracing::debug!("getting thread…");
  let thread = Thread::get("2ch.hk", board, thread.id).await?;
  let posts = thread.posts.iter();
  let (post, _) = posts
    .scan(0, |acc, thread| {
      *acc += thread.files.as_deref().map_or(0, |files| files.len());
      Some((thread, *acc))
    })
    .find(|&(_, acc)| file_index < acc)
    .unwrap();

  reply(ctx, "2ch.hk", board, post.id, &thread).await
}

// ---

async fn reply(ctx: &Context<'_>, domain: &str, board_id: &str, post_id: u64, thread: &Thread) -> Result<()> {
  let (post, post_index) = thread.get_post_by_id(post_id).unwrap();
  let op = &thread.posts[0];

  let url = format!("https://{}/{}/res/{}.html#{}", domain, board_id, op.id, post_id);
  let comment = html::strip(&op.comment);

  let chars = (op.subject.chars(), comment.chars());
  let title = match iter::zip(chars.0, chars.1).find(|&(c0, c1)| c0 != c1) {
    Some((' ', '\n')) | None => &comment,
    Some(_) => &op.subject,
  };
  let title = if title.is_empty() { "<no subject>" } else { title };
  let title = fmt::ellipsis(title, 100);
  let title = fmt::line_ellipsis(&title, 2);

  let content = post.render(domain);
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

  let mut files = EditAttachments::new();
  if let Some(post_files) = &post.files {
    tracing::debug!("attaching files…");
    for file in post_files {
      let url = format!("https://{}{}", domain, file.path);
      let att = CreateAttachment::url(ctx, &url).await?;
      files = files.add(att);
    }
  }

  tracing::debug!("sending response…");
  let edit = EditInteractionResponse::new().attachments(files);
  ctx.event.edit_response(ctx, edit.embed(embed)).await?;

  Ok(())
}
