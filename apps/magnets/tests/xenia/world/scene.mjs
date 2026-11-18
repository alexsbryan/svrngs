// Magnets' scenes for xenia: the stand-in (apps/magnets/tests/standin.mjs)
// serving the fridge on this computer, and the scene line its name asks for
// (xenia DESIGN §5, "Scene"), held until xenia closes stdin. Then the board
// each person left, on stderr, for the run's record.

import { serve } from '../../standin.mjs';

const standin = await serve();
const [at, origin] = [standin.at, new URL(standin.at).origin];
const fridge = { name: 'the fridge', url: '/' };
// A move refused, the board shut or unreachable: the stand-in says none of
// these unless a test switches it, so any one seen is a fault.
const faults = [
  {
    expr: "document.querySelector('#said[data-state=refused], #said[data-state=shut], #said[data-state=unreachable]') !== null",
  },
];

const SCENES = {
  // Proof 1's first half: the open fridge on a phone, nothing said about it.
  'first-move': {
    begins: 'on a phone, their group’s Magnets already open; nobody told them anything about it',
    cast: [
      {
        as: 'Player',
        device: 'phone',
        url: at,
        goal: { expr: "document.querySelectorAll('#placed button').length > 0" },
      },
    ],
    screens: [fridge],
    faults,
    reach: [origin],
  },
  // Its second half: alone on the board, with the house chat a message away.
  // Whether they send it is what they do, unprompted; what it is for is what
  // they tell.
  solo: {
    begins:
      'on a phone, in a group of one they just made, Magnets just brought to it; their housemates are in the house chat',
    explore: {
      purpose:
        'A fridge of word magnets a group shares: you make a funny sentence, and anyone in the group can change it.',
    },
    cast: [{ as: 'Player', device: 'phone', url: at }],
    screens: [fridge],
    channels: [
      {
        name: 'the house chat',
        members: ['Player'],
        opening: [{ from: 'Sam', text: 'Dinner’s at 7 tomorrow, I’m cooking.' }],
      },
    ],
    faults,
    reach: [origin],
  },
};

const name = process.argv[2];
if (!(name in SCENES)) {
  console.error(
    `no Magnets scene ${JSON.stringify(name)}: one of ${Object.keys(SCENES).join(', ')}`,
  );
  process.exit(2);
}
console.log(JSON.stringify(SCENES[name]));
process.stdin.resume();
process.stdin.on('end', async () => {
  const [{ rows }] = await standin.db.run([
    { sql: 'SELECT id, position FROM magnets ORDER BY position, id' },
  ]);
  console.error(`magnets ${name}: the board left ${JSON.stringify(rows)}`);
  standin.stop();
  process.exit(0);
});
