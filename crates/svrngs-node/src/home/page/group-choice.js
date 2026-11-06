// SPDX-License-Identifier: AGPL-3.0-or-later
// Who can open a lend: one box a group of HOME's, each by its stable id,
// ticked when it is the lend's, so a lend is made to the groups ticked and
// never to everyone by default (SCOPE_PARALLEL.md, "Shared API shape").
// The same boxes draw a new lend (found.js) and a change of one
// (lent-row.js), each sending `choose` with the group's id and whether it
// was ticked.
import { html } from '../ui/view.js?v=__VERSION__';
import { whose } from '../question.js?v=__VERSION__';

/**
 * A group a lend can be made to, by HOME: its stable id, its name, and who
 * made it, so two groups of one name are told apart (group.js `Group`).
 * @typedef {object} Choice
 * @property {string} id
 * @property {string | null} name
 * @property {{ name: string, maker: string | null, since: number | null } | null} [made]
 */

/**
 * The boxes' props: HOME's groups, the ids ticked, whether they wait on a
 * change under way, and where a tick is sent.
 * @typedef {object} GroupChoiceProps
 * @property {Choice[]} groups
 * @property {string[]} ticked
 * @property {boolean} [waiting]
 * @property {(message: { type: 'choose', id: string, on: boolean }) => void} send
 */

/**
 * A group as the boxes and a lend's row both say it: its name, and who made
 * it, so two of one name are told apart.
 * @param {Choice} group
 */
export const called = (group) => {
  const made = group.made && whose(group.made);
  return `${group.name ?? 'a group whose name this computer cannot read'}${made ? ` (${made})` : ''}`;
};

/** @param {GroupChoiceProps} props */
export function GroupChoice({ groups, ticked, waiting = false, send }) {
  /** @param {string} id */
  const tick = (id) => (/** @type {Event} */ pressed) =>
    send({
      type: 'choose',
      id,
      on: /** @type {HTMLInputElement} */ (pressed.currentTarget).checked,
    });
  return groups.map((group) => {
    return html`
      <p>
        <label>
          <input
            type="checkbox"
            disabled=${waiting}
            checked=${ticked.includes(group.id)}
            onChange=${tick(group.id)}
          />
          ${called(group)}
        </label>
      </p>
    `;
  });
}
