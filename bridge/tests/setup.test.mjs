// What setup tells a person about their Discord application, without Discord.
import assert from 'node:assert/strict';
import { test } from 'node:test';

import { ApplicationFlags } from 'discord.js';

import { inviteLink, switchesOff } from '../discord.mjs';

test('the invite adds the bot with its commands, and asks only to see, read and post', () => {
  const url = new URL(inviteLink('123'));
  assert.equal(url.searchParams.get('client_id'), '123');
  assert.equal(url.searchParams.get('scope'), 'bot applications.commands');
  // ViewChannel 1024 + SendMessages 2048 + ReadMessageHistory 65536 + UseApplicationCommands 2^31.
  assert.equal(url.searchParams.get('permissions'), String(1024 + 2048 + 65536 + 2 ** 31));
});

test('each privileged switch that is off is named, and a limited flag counts as on', () => {
  assert.deepEqual(switchesOff(0), ['Server Members Intent', 'Message Content Intent']);
  assert.deepEqual(switchesOff(ApplicationFlags.GatewayGuildMembersLimited), ['Message Content Intent']);
  assert.deepEqual(
    switchesOff(ApplicationFlags.GatewayGuildMembers | ApplicationFlags.GatewayMessageContentLimited),
    [],
  );
});
