// The Discord adapter: discord.js 14 on the gateway, so it only dials out
// (DESIGN C5). Everything Discord-specific lives here, behind the five calls
// bridge.mjs names. The application needs the Server Members and Message
// Content intents switched on in Discord's developer portal.

import {
  ActionRowBuilder,
  ButtonBuilder,
  ButtonStyle,
  Client,
  Events,
  GatewayIntentBits,
  MessageFlags,
} from 'discord.js';

import { log } from './bridge.mjs';

/** A Discord user as the bridge names people. */
function person(user, member) {
  return { speaker: `discord:${user.id}`, name: member?.displayName ?? user.globalName ?? user.username };
}

/** Mentions are suppressed, except the people a message names on purpose (C5). */
function render({ text, buttons = [], mention = [] }) {
  const out = {
    content: text,
    allowedMentions: { parse: [], users: mention.map((s) => s.replace(/^discord:/, '')) },
  };
  if (buttons.length) {
    const row = new ActionRowBuilder().addComponents(
      buttons.map((b) => new ButtonBuilder().setCustomId(b.id).setLabel(b.label).setStyle(ButtonStyle.Primary)),
    );
    out.components = [row];
  }
  return out;
}

export function discordAdapter({ token, channelId }) {
  const client = new Client({
    intents: [
      GatewayIntentBits.Guilds,
      GatewayIntentBits.GuildMembers,
      GatewayIntentBits.GuildModeration,
      GatewayIntentBits.GuildMessages,
      GatewayIntentBits.MessageContent,
    ],
  });
  let channel = null;
  // What a reply answers, by interaction id: the platform's own object, kept
  // as long as Discord lets an interaction be answered (fifteen minutes).
  const pending = new Map();
  const hold = (id, target) => {
    pending.set(id, target);
    setTimeout(() => pending.delete(id), 15 * 60 * 1000).unref();
  };
  const ours = (guildId) => channel && guildId === channel.guildId;

  return {
    async connect(on) {
      client.on(Events.GuildMemberAdd, (m) => ours(m.guild.id) && !m.user.bot && on.joined(person(m.user, m)));
      client.on(
        Events.GuildMemberRemove,
        (m) => ours(m.guild.id) && !m.user.bot && on.left({ ...person(m.user, m), why: 'left' }),
      );
      client.on(Events.GuildBanAdd, (b) => ours(b.guild.id) && on.left({ ...person(b.user), why: 'banned' }));
      client.on(Events.MessageCreate, (msg) => {
        if (msg.channelId !== channelId || msg.author.bot) return;
        hold(msg.id, msg);
        on.input({
          kind: 'message',
          ...person(msg.author, msg.member),
          interaction: msg.id,
          text: msg.content,
          mentions: [...msg.mentions.users.values()].filter((u) => !u.bot).map((u) => person(u)),
        });
      });
      client.on(Events.InteractionCreate, (i) => {
        if (i.channelId !== channelId || !(i.isChatInputCommand() || i.isButton())) return;
        hold(i.id, i);
        on.input({
          kind: i.isButton() ? 'button' : 'command',
          ...person(i.user, i.member),
          interaction: i.id,
          command: i.isChatInputCommand() ? i.commandName : undefined,
          options: i.isChatInputCommand() ? i.options.data.map((o) => ({ name: o.name, value: o.value })) : undefined,
          button: i.isButton() ? i.customId : undefined,
          mentions: [],
        });
      });
      const ready = new Promise((resolve) => client.once(Events.ClientReady, resolve));
      await client.login(token);
      await ready;
      channel = await client.channels.fetch(channelId);
      if (!channel?.isTextBased()) throw new Error(`channel ${channelId} is not a text channel this bot can see`);
      log('bridge:discord_ready', { channel: channelId, guild: channel.guildId });
    },

    async post(message) {
      await channel.send(render(message));
    },

    async reply(input, message) {
      const target = pending.get(input.interaction);
      pending.delete(input.interaction);
      if (!target) throw new Error(`no ${input.kind} ${input.interaction} to answer`);
      const out = render(message);
      if (message.private && input.kind !== 'message') out.flags = MessageFlags.Ephemeral;
      await target.reply(out);
    },

    async commands(schemas) {
      await channel.guild.commands.set(schemas);
    },

    async close() {
      await client.destroy();
    },
  };
}
