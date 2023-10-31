use std::collections::HashMap;

use ::serenity::all as serenity;

use super::{Command, CommandOption};

pub(crate) use macros::*;

pub type Commands = HashMap<&'static str, CommandTree>;

#[derive(Debug)]
pub enum CommandTree {
  Command(Command),
  Commands(Commands),
}

pub fn serialize(commands: &Commands) -> Vec<serenity::CreateCommand> {
  fn serialize_command_tree(tree: &CommandTree, name: &str) -> serenity::CreateCommandOption {
    match tree {
      CommandTree::Command(cmd) => serialize_subcommand(cmd),
      CommandTree::Commands(cmds) => {
        let ty = serenity::CommandOptionType::SubCommandGroup;
        let builder = serenity::CreateCommandOption::new(ty, name, "…");
        cmds.iter().fold(builder, |b, (&name, tree)| {
          let opt = serialize_command_tree(tree, name);
          b.add_sub_option(opt)
        })
      }
    }
  }

  fn serialize_commands(cmds: &Commands, name: &str) -> serenity::CreateCommand {
    let builder = serenity::CreateCommand::new(name).description("…");
    cmds.iter().fold(builder, |b, (&name, tree)| {
      let opt = serialize_command_tree(tree, name);
      b.add_option(opt)
    })
  }

  fn serialize_subcommand(cmd: &Command) -> serenity::CreateCommandOption {
    let ty = serenity::CommandOptionType::SubCommand;
    let desc = cmd.description.unwrap_or("…");
    let builder = serenity::CreateCommandOption::new(ty, cmd.name, desc);
    cmd.options.iter().fold(builder, |b, &opt| {
      let opt = serialize_command_option(opt);
      b.add_sub_option(opt)
    })
  }

  fn serialize_command(cmd: &Command) -> serenity::CreateCommand {
    let desc = cmd.description.unwrap_or("…");
    let builder = serenity::CreateCommand::new(cmd.name).description(desc);
    cmd.options.iter().fold(builder, |b, &opt| {
      let opt = serialize_command_option(opt);
      b.add_option(opt)
    })
  }

  fn serialize_command_option(opt: &CommandOption) -> serenity::CreateCommandOption {
    let choices = opt.choices.unwrap_or(&[]).iter();
    let desc = opt.description.unwrap_or("…");
    let builder = serenity::CreateCommandOption::new(opt.ty, opt.name, desc).required(opt.required);
    choices.fold(builder, |b, ch| b.add_string_choice(ch.name, ch.value))
  }

  let commands = commands.iter().map(|(&name, tree)| match tree {
    CommandTree::Command(cmd) => serialize_command(cmd),
    CommandTree::Commands(cmds) => serialize_commands(cmds, name),
  });

  commands.collect()
}

pub fn resolve<'a>(
  commands: &'a Commands,
  options: Vec<serenity::ResolvedOption<'a>>,
  name: &str,
) -> (&'a Command, Vec<serenity::ResolvedOption<'a>>) {
  use serenity::all::{ResolvedOption, ResolvedValue::*};

  match commands.get(name).unwrap() {
    CommandTree::Command(command) => (command, options),
    CommandTree::Commands(commands) => {
      let ResolvedOption { name, value, .. } = options.into_iter().next().unwrap();
      let (SubCommand(options) | SubCommandGroup(options)) = value else {
        unreachable!()
      };
      resolve(commands, options, name)
    }
  }
}

mod macros {
  macro_rules! commands {
    ($acc:expr,) => {};
    ($acc:expr, $name:expr => $command:path, $($tokens:tt)*) => {
      $acc.insert($name, $crate::client::CommandTree::Command($command($name)));
      $crate::client::commands!($acc, $($tokens)*);
    };
    ($acc:expr, $name:expr => { $($subcommands:tt)* }, $($tokens:tt)*) => {
      $acc.insert($name, $crate::client::CommandTree::Commands({
        $crate::client::commands!($($subcommands)*)
      }));
      $crate::client::commands!($acc, $($tokens)*);
    };
    ($($tokens:tt)*) => {{
      let mut acc = $crate::client::Commands::new();
      $crate::client::commands!(&mut acc, $($tokens)*);
      acc
    }};
  }

  pub(crate) use commands;
}
