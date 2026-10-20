// The doc's one bundle: what its page and its server import, built by
// build.sh. One copy of Yjs for everything that uses it, so the editor's
// binding, the browser's copy and the sync share one Y.
export * as Y from 'yjs';
export * as awarenessProtocol from 'y-protocols/awareness';
export * as syncProtocol from 'y-protocols/sync';
export * as encoding from 'lib0/encoding';
export * as decoding from 'lib0/decoding';
export { IndexeddbPersistence } from 'y-indexeddb';
export { Editor } from '@tiptap/core';
export { Document } from '@tiptap/extension-document';
export { Paragraph } from '@tiptap/extension-paragraph';
export { Text } from '@tiptap/extension-text';
export { Bold } from '@tiptap/extension-bold';
export { Italic } from '@tiptap/extension-italic';
export { Heading } from '@tiptap/extension-heading';
export { BulletList, ListItem } from '@tiptap/extension-list';
export { HardBreak } from '@tiptap/extension-hard-break';
export { Collaboration } from '@tiptap/extension-collaboration';
export { CollaborationCaret } from '@tiptap/extension-collaboration-caret';
