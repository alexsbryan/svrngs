// A step's page errors, judged by the page they came from, never by their
// words (demo/runs/flakes/2026-11-12, holyarrs H8). A browser reports the
// loads a navigation away cancels, and the reads a removal cuts, as page
// errors, each engine in its own words: WebKit's "Load failed", its access
// control message, a bundler's ChunkLoadError. A list of those words grows
// with every engine and app. So a scene says whose page each step leaves
// (navigates away from, or cuts off on purpose), and only an error from a
// page its step leaves goes unjudged; every other is judged, whatever it says.

/**
 * @param {{ who: string, at: string }[]} errors each with whose page and which step
 * @param {Map<string, string[]>} leaves each step's name, and whose pages it leaves
 */
export function leaving(errors, leaves) {
  const left = (e) => leaves.get(e.at)?.includes(e.who) ?? false;
  return { left: errors.filter(left), judged: errors.filter((e) => !left(e)) };
}
