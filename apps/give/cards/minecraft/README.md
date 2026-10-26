# Minecraft

A world your group plays in, kept by one of your computers at a time. Each
player needs Minecraft: Java Edition, at the version the card names (26.3).
In the launcher, pick it under Installations.

Minecraft's makers ask whoever runs a server to agree to their terms, the
Minecraft EULA. Give asks the person who keeps the world, and asks again
whoever keeps it next.

The card runs `itzg/minecraft-server`, pinned by digest. Its `VERSION` is
pinned because the image's default is the latest release. The world is the
image's `/data`, and health is the image's own `mc-health`.
