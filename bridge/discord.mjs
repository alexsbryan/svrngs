// The Discord adapter: discord.js 14 on the gateway, so it only dials out
// (tag e0-bot, DESIGN C5; next, the Discord door, DESIGN U4). Everything Discord-specific lives here, behind the five calls
// bridge.mjs names. The application needs the Server Members and Message
// Content intents switched on in Discord's developer portal.

import {
  ActionRowBuilder,
  ApplicationCommandOptionType,
  ApplicationFlags,
  ButtonBuilder,
  ButtonStyle,
  ChannelType,
  Client,
  DiscordAPIError,
  Events,
  GatewayIntentBits,
  MessageFlags,
  PermissionFlagsBits,
  REST,
  Routes,
} from 'discord.js';

import { log } from './bridge.mjs';

/** A Discord user as the bridge names people. */
function person(user, member) {
  return { speaker: `discord:${user.id}`, name: member?.displayName ?? user.globalName ?? user.username };
}

/** How a command option kind the host declares becomes Discord's. */
const OPTION = { text: ApplicationCommandOptionType.String, person: ApplicationCommandOptionType.User };

/** A command an app declares, as Discord registers it. */
function command({ name, description, options }) {
  return {
    name,
    description,
    options: options.map((o) => {
      if (!(o.kind in OPTION)) throw new Error(`/${name} ${o.name}: no Discord option for the kind ${o.kind}`);
      return { name: o.name, description: o.description, required: o.required, type: OPTION[o.kind] };
    }),
  };
}

/** Mentions are suppressed, except the people a message names on purpose (C5). */
function render({ text, buttons = [], mention = [] }) {
  const users = mention.map((s) => s.replace(/^discord:/, ''));
  const out = {
    content: [...users.map((u) => `<@${u}>`), text].join(' '),
    allowedMentions: { parse: [], users },
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
        const options = i.isChatInputCommand() ? i.options.data : [];
        const isUser = (o) => o.type === ApplicationCommandOptionType.User;
        on.input({
          kind: i.isButton() ? 'button' : 'command',
          ...person(i.user, i.member),
          interaction: i.id,
          command: i.isChatInputCommand() ? i.commandName : undefined,
          options: Object.fromEntries(options.map((o) => [o.name, isUser(o) ? `discord:${o.value}` : o.value])),
          button: i.isButton() ? i.customId : undefined,
          mentions: options.filter((o) => isUser(o) && o.user && !o.user.bot).map((o) => person(o.user, o.member)),
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
      await channel.guild.commands.set(schemas.map(command));
    },

    async close() {
      await client.destroy();
    },
  };
}

/** What the bot asks for in a server: to see the channel, read and post in it, and take commands. */
const PERMISSIONS =
  PermissionFlagsBits.ViewChannel |
  PermissionFlagsBits.SendMessages |
  PermissionFlagsBits.ReadMessageHistory |
  PermissionFlagsBits.UseApplicationCommands;

/**
 * The privileged intents `discordAdapter` asks for, each by the label of its
 * switch in the developer portal. An unverified application gets the
 * `Limited` flag when its switch is on.
 */
const SWITCHES = [
  { label: 'Server Members Intent', flags: ['GatewayGuildMembers', 'GatewayGuildMembersLimited'] },
  { label: 'Message Content Intent', flags: ['GatewayMessageContent', 'GatewayMessageContentLimited'] },
];

/** The link that adds application `id` to a server, with the commands and the permissions above. */
export function inviteLink(id) {
  return `https://discord.com/oauth2/authorize?client_id=${id}&scope=bot+applications.commands&permissions=${PERMISSIONS}`;
}

/** The switches still off, by label, in an application's `flags`. */
export function switchesOff(flags) {
  return SWITCHES.filter((s) => !s.flags.some((f) => flags & ApplicationFlags[f])).map((s) => s.label);
}

/**
 * What setup asks Discord with the bot's token (tag e0-bot, DESIGN C11): the
 * application it belongs to, and the text channels in the servers it is in.
 * A token Discord refuses comes back as `{refused: 'token'}`, not a throw.
 */
export function discordSetup({ token }) {
  const rest = new REST({ version: '10' }).setToken(token);
  const ask = async (fn) => {
    try {
      return await fn();
    } catch (err) {
      if (err instanceof DiscordAPIError && err.status === 401) {
        log('bridge:setup_token_refused', {});
        return { refused: 'token' };
      }
      throw err;
    }
  };
  return {
    application: () =>
      ask(async () => {
        const app = await rest.get(Routes.currentApplication());
        return {
          id: app.id,
          name: app.name,
          owner: app.owner?.global_name ?? app.owner?.username ?? null,
          switches_off: switchesOff(app.flags ?? 0),
          settings: `https://discord.com/developers/applications/${app.id}/bot`,
          invite: inviteLink(app.id),
        };
      }),
    servers: () =>
      ask(async () => {
        const servers = [];
        for (const g of await rest.get(Routes.userGuilds())) {
          const channels = (await rest.get(Routes.guildChannels(g.id)))
            .filter((c) => c.type === ChannelType.GuildText)
            .sort((a, b) => a.position - b.position)
            .map((c) => ({ id: c.id, name: c.name }));
          servers.push({ id: g.id, name: g.name, channels });
        }
        return { servers };
      }),
  };
}
