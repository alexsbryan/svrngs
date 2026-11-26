# Local Hosting Inspiration

What could your group run on the computers it already has? Start with
something you’d use together. These ideas mix [apps that come with svrngs](apps.md)
with third-party programs you set up yourself. If you have something running
already, [lend it to your group](lend.md) and send a friend the link.

## Watch, listen, and remember

- **[Movie Night](../../apps/movie-night/README.md).** Search two friends’
  Jellyfin libraries from one page and pick whose copy to play. Use Events for
  RSVPs and Money for the pizza. Start with a clip you can share.
- **[Can We Watch This?](../../demo/holyarrs.md)** Ask for a film in HolyArrs.
  A librarian approves the request; when the film is in their Jellyfin
  library, it shows up in Movie Night.
- **[What Should We Watch?](../../apps/taste/README.md)** Rate films you’ve seen
  and get recommendations from your friends’ libraries. Taste sends the film
  catalog to a friend’s model with your permission and keeps your ratings in
  your browser.
- **[Listening Room](lend.md#lend-navidrome-with-nobody-signing-in).** Lend
  Navidrome so friends can listen to your music. It can take their identity
  from svrngs so they don’t need a separate password.
- **Reading Room.** Lend [Calibre-Web](lend.md#lend-calibre-web) for ebooks or
  [Audiobookshelf](https://www.audiobookshelf.org/) for audiobooks. Give a book
  club one place to find the next read.
- **The Wedding Album.** Invite friends with Immich accounts to contribute to
  an album you host. Use a [shared folder](folders.md#use-your-groups-folder)
  when you want everyone to keep their own copies on disk.
- **[The Band’s Folder](folders.md#share-another-folder).** Share rehearsal
  takes, stems, and setlists. If two people change a setlist before syncing,
  the folder keeps both versions so the band can decide what to use.

## Borrow compute and tools

- **House AI.** Put [Open WebUI](https://docs.openwebui.com/) in front of
  [llama-server](models.md#lend-your-llama-server) and lend the chat page. A
  friend can chat from a phone using the GPU in your closet. Brave Search adds
  web answers; SearXNG is an option if you want to run the search service too.
- **[Research Night](https://docs.openwebui.com/features/chat-conversations/web-search/agentic-search/).**
  Use Open WebUI’s search and page fetching with a model that calls tools
  reliably. Ask it to compare three sources, then open the citations together.
- **[Borrow a Coder](models.md#use-a-friends-model-from-a-program).** Press
  **Play** on a friend’s model and point Aider or OpenCode at its address. Your
  editor stays on your computer; their model does the inference.
- **[The GPU Co-op](../../apps/pantry/README.md).** Use Pantry to put several
  friends’ models in one picker. Pick whose model answers each request. You
  can see who’s available without combining everyone’s GPUs into a cluster.
- **[Voice Studio](../../apps/pantry/README.md#recipes-two-shell-lines-per-server).**
  Transcribe a rehearsal using one friend’s whisper.cpp server and audition
  narration using another friend’s Kokoro server.
- **[Spot Me a Model](../../apps/spot/README.md).** Give your group a night on
  a paid model. Set a token budget and an end time. Friends can chat without
  getting your provider key.
- **[Borrow a Tool](models.md#the-pantry-every-friends-model-behind-one-address).**
  Lend a read-only MCP tool that looks up the workshop inventory. Someone’s
  chat app can ask who has the projector; the lookup runs on your computer.
- **GPU Art Room.** Try lending [ComfyUI](https://www.comfy.org/) for an
  image-making session. This is an experiment: see how sharing the interface
  and waiting for each other’s jobs feels on one GPU.

## Run a house, club, or project

- **[Weekend Away](apps.md).** Post the itinerary in Events, split expenses
  with Money, and put the photos in the shared folder. Everyone confirms
  their own share of the bill.
- **House Book.** Open a budget together in [Actual Budget](https://actualbudget.org/);
  it lives on the host. For a simple record of who owes whom that the group
  keeps, use [Money](apps.md#money-split-a-bill).
- **Kitchen.** Try [Mealie](https://mealie.io/) for a shared recipe box or
  [Grocy](https://grocy.info/) to track what’s in the pantry. Pick what to
  cook from what’s left, then invite everyone to dinner in Events.
- **[House Meeting](apps.md#house-rules-agree-how-your-house-runs).** Choose
  how your house makes decisions, propose one, and let people agree or object.
  Keep the meeting notes in Doc and put them on the projector.
- **[Writers’ Room](../../apps/doc/README.md).** Write together with named
  carets. If the host briefly goes away, keep typing; the browsers send their
  edits back when it returns.
- **[The Filing Cabinet](../../demo/runs/catalogue/2026-10-05/RESULTS.md).**
  Upload a receipt from your phone to a friend’s Paperless-ngx for OCR and
  filing. Try Nextcloud for ordinary browser uploads and downloads.
- **[Make an App for This Room](build.md#a-guestbook).** Turn the guestbook
  example into a volunteer rota. Put its folder in the group, then have a
  friend open it on a phone and sign up for Saturday.
- **[Show Me Your Branch](lend.md).** Lend a web preview you already have on
  localhost. A teammate tries the actual running branch on their phone
  before you deploy it anywhere.

## Play, host, and reach past the house

- **[Minecraft Night / Factorio Night](games.md).** Lend a Minecraft server
  over TCP or a Factorio game over UDP. Friends join from their normal game
  clients using the address svrngs gives them.
- **[Keep the World](games.md#keep-a-game-for-the-group-with-give).** With
  Give, hand a supported game to another member who agrees to host it.
  Rejoin and look for the chest you left behind. If the host disappears,
  someone can take over from the last available copy.
- **[The House on the Big Screen](tv.md).** Put the group page on a projector.
  Or, on trusted Wi-Fi, open a friend’s Jellyfin from the TV’s own app. If you
  try it on a real TV, tell us which one and how it went.
- **[The Guest Pass](people.md#have-a-guest).** Let a weekend guest watch
  something you lend without joining your group or seeing its records. End
  the visit when they go home.
- **[Borrow a Projector](../../demo/grapevine.md).** Ask the house if anyone
  has a projector. Someone in both the house and the neighborhood carries
  the ask across and brings back an answer without either group joining the
  other.
- **[The Spare Lamp](../../demo/runs/catalogue/2026-10-05/RESULTS.md).** Lend
  a Home Assistant dashboard. Have a friend open it from a phone and watch
  the lamp’s state change live.
- **[The Internet Went Out](../../demo/mesh-house.md).** Try a local relay,
  two laptops, a shared folder, and a model on the same mesh network. Cut the
  uplink and see what still works between those computers.
- **[Run the Door Yourself](self-host.md).** Host your own group page,
  downloads, and relay. Give a friend an invitation from infrastructure
  your group runs.
- **[House Radio](../../apps/on-air/README.md).** Try the On Air prototype:
  a friend broadcasts a set and you listen in a browser.

## Tell us what you tried

[Open an issue](https://github.com/alexsbryan/svrngs/issues/new) and tell us
what you ran, what your friends did with it, and where you got stuck. If
they asked to use it again, we especially want to hear why. That’s what we’ll
use to decide which ideas get tested setup guides or an easier install.
