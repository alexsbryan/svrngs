// SPDX-License-Identifier: AGPL-3.0-or-later
// The home's v2 page, its one root (page-scaffold (f)): served at
// /v2/home.js to a test build's node chosen v2 (home/v2/serve.rs), and
// drawn into home/v2/index.html's #home. It is empty until page-open draws
// a person's groups in it; until then it says what v1's page says before
// its first read (home/index.html), so the page is never blank. The
// literal ?v=__VERSION__ is left unstamped here, one URL per module still.
import { trace } from '../ui/trace.js?v=__VERSION__';
import { html, render } from '../ui/view.js?v=__VERSION__';

export const Home = () => html`<p class="said">Reading your groups…</p>`;
render(html`<${Home} />`, document.getElementById('home'));
trace('HOME: drawn, with nothing read yet');
