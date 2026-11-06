# Jellyfin

For whoever lends Jellyfin. svrngs finds it by its own port, 8096:
`svrngs share 8096 --group <group>`, or lend it from the home page under "Already running
here". It is lent as it comes, with no password typed, so Jellyfin asks
each person to sign in once, with an account you make them; `share` tells
you how, and the catalogue's run measured it on one computer
(`demo/runs/catalogue/2026-10-05/`).

## Set up Sign in with svrngs

Turning on **Sign people in with svrngs**, or running
`svrngs share 8096 --sign-in-with-svrngs --group <group>`, creates sign-in credentials.
**It does not install a Jellyfin plugin or configure Jellyfin.** Complete
the steps below on the computer lending Jellyfin, using its administrator
account. Keep an ordinary administrator login available while testing.

### 1. Check the plugin and install it

Jellyfin does not have a stock OpenID Connect settings form. It needs the
third-party [SSO Authentication (SSO-Auth) plugin][plugin]. Upstream is
**archived**, as checked on 2026-11-04; this is not a Jellyfin-maintained
integration. The successful [recorded trial][trial] used **Jellyfin 10.11.11
and SSO-Auth 4.0.0.4**. The current [release manifest][manifest] lists that
plugin version for ABI **10.11.0.0**. That is compatibility metadata, not
a successful test of every Jellyfin version; other versions need their own
verification. The trial covers browsers on computers and phones, not every
native Jellyfin client.

1. In Jellyfin's **Dashboard → Plugins → Repositories**, add a repository
   named `SSO-Auth` with this URL:
   `https://raw.githubusercontent.com/9p4/jellyfin-plugin-sso/manifest-release/manifest.json`.
2. Under **Plugins → Catalog**, install **SSO Authentication**, choosing
   **4.0.0.4** for the tested Jellyfin 10.11 pairing, then restart Jellyfin.
3. Open **Dashboard → Plugins → My Plugins → SSO Authentication → Settings**
   (the plugin's configuration page is titled **SSO Settings**). If the
   plugin is missing or incompatible, stop here and check Jellyfin's plugin
   status/logs rather than assuming the svrngs switch completed setup.

### 2. Create credentials and check the server can reach the issuer

On the svrngs home page, turn on **Sign people in with svrngs** for this
lend, or use the command above. Copy the issuer, client id and secret into
the plugin form before reloading the page: svrngs does not show the secret
on a fresh page load. Do not paste it into chat, a bug report or a URL.

The issuer is exactly `http://127.0.0.1:<svrngs-api-port>/oidc/<lend-name>`,
with **no trailing slash**. Copy the actual value printed for your lend;
8096 is Jellyfin's port, not the svrngs API port. Discovery lives at
`<issuer>/.well-known/openid-configuration`. From the **Jellyfin server's
own environment**, opening that address must return discovery with the
same `issuer`. Token and key requests also come from that server, not the
attendee's browser or computer. Keep svrngs running.

**Containers:** `127.0.0.1` inside a container is the container, not its host.
The tested trial ran Jellyfin and svrngs on the same host loopback. A
host-gateway name is not a drop-in replacement: svrngs's back channel is
loopback-only and the issuer must match the token exactly. If discovery
cannot be reached from Jellyfin, this deployment is not ready for this
setup. Use a deployment sharing the lender's loopback (for example a native
Jellyfin server); do not expose the record API on the network or disable
issuer validation to work around it. No container-network pairing is
claimed tested here.

### 3. Fill in the plugin's OpenID provider

In **SSO Settings → Add / Update Provider Configuration**, use the
following mapping. Names in parentheses are the plugin's configuration
keys, also useful when checking a saved configuration.

| Plugin field | Value |
|---|---|
| Name of OpenID Provider | `svrngs` (this local provider name is used by the login button; it is not the generated client id) |
| OpenID Endpoint (`OidEndpoint`) | The exact issuer printed by svrngs |
| OpenID Client ID (`OidClientId`) | The generated client id |
| OpenID client secret (`OidSecret`) | The generated secret |
| Enabled | Checked |
| Set default username claim (`DefaultUsernameClaim`) | `preferred_username`; do not substitute the display `name`, which two people may share |
| Request Additional Scopes (`OidScopes`) | Empty; no email scope (svrngs supplies no email address) |
| Disable OpenID HTTPS Discovery (`DisableHttps`) | Checked: this issuer is HTTP on the lender's loopback only |
| Do Not Validate OpenID Endpoints (`DoNotValidateEndpoints`) | Checked: discovery deliberately puts the browser authorize endpoint on the lend's origin and the back channel on the lender's API |
| Do Not Validate OpenID Issuer Name (`DoNotValidateIssuerName`) | **Unchecked**: keep issuer validation |
| Disable Pushed Authorization (`DisablePushedAuthorization`) | Checked, as in the trial; svrngs advertises no PAR endpoint |
| Do Not Load Profile Information (`DoNotLoadProfile`) | Unchecked |
| Scheme Override / Port Override | Empty; let the trusted proxy headers describe the browser's origin |
| Set default Provider / avatar URL format | Empty |

**Choose access deliberately.** For provider-managed permissions, check
**Enable Authorization by Plugin**, leave **Enable All Folders unchecked**,
and select only the libraries you intend to share under **Enabled Folders**.
Leave **Admin Roles empty**, **Roles empty**, and **Role Claim empty**; leave
role-based folder access, Live TV access/management and their role options
off. This does not grant administrator rights. The old trial enabled all
folders for its disposable fixture; that is not a default to copy onto your
server. Alternatively, leave plugin authorization off and explicitly manage
each new user's access in Jellyfin's **Dashboard → Users** before testing
playback. Inspect the resulting permissions either way: upstream documents
that plugin-managed permissions can overwrite an existing matching account.

Press **Save**, reload and use **Select Existing Provider to Modify →
svrngs → Load Provider** to check the saved values. Restart Jellyfin after
configuration changes, as the plugin's configuration page instructs.

### 4. Trust the local proxy and add a login button

1. In Jellyfin's **Dashboard → Networking → Known Proxies**, add
   `127.0.0.1` for the tested same-host arrangement, preserving any existing
   entries you still need, then save and restart. This lets Jellyfin use
   svrngs's `X-Forwarded-Host` and `X-Forwarded-Proto` for the callback at
   the browser's lent-app origin. Do not trust every proxy. A different
   deployment must establish the actual proxy address it sees; the trial
   does not establish one for a container.
2. In **Dashboard → General → Branding → Login disclaimer**, add:

   ```html
   <form action="/sso/OID/start/svrngs">
     <button class="raised block emby-button button-submit">Sign in with svrngs</button>
   </form>
   ```

   The relative action keeps the attendee on their lent-app origin. If you
   chose another provider name, replace the final `svrngs` with that name,
   not with the generated client id. The matching callback is
   `/sso/OID/redirect/svrngs` on that same origin; do not send the attendee
   to the lender's `127.0.0.1` or hard-code one attendee's address.
3. In Branding's **Custom CSS code**, add the upstream visibility rule
   `.disclaimerContainer { display: block; }` if needed to show the button.
   Save, then reload the login page.

### 5. Verify the whole path

Use a group member's fresh browser session and open Jellyfin **from the
group's app list**, not directly on port 8096. Press **Sign in with
svrngs**. There should be no password or email to type; after the callback,
Jellyfin should show that person's own account. In **Dashboard → Users**,
verify its unique username, that it is **not an administrator**, and that
only the selected libraries are available. Play an item the account is
allowed to see. Repeat as a second member in a separate browser session:
the two accounts must differ and one sign-in must not replace the other.
The recorded trial did this in Chromium and WebKit; this guide update is
not a new real-server test.

- **No button:** check the plugin is loaded, the provider name in the form,
  the Branding save, and the disclaimer visibility rule.
- **Discovery/token failure:** test discovery from the Jellyfin server's
  environment, check svrngs is running and the printed issuer/client values
  are current. Container localhost and an API port changed since setup are
  different from an ended group invitation.
- **Stuck on “Logging in…” or redirected to the wrong computer:** recheck
  Known Proxies and the callback origin. The first trial failed here until
  the same-host proxy setting was added; keep issuer validation enabled.
- **Signed in but no media:** inspect that user's library permissions and
  your explicit selection; do not solve this by granting all libraries or
  administrator access.

svrngs does not detect completion of this app-side setup. Its saved-client
message remains a setup reminder, not evidence of a successful login.

[plugin]: https://github.com/9p4/jellyfin-plugin-sso
[manifest]: https://raw.githubusercontent.com/9p4/jellyfin-plugin-sso/manifest-release/manifest.json
[trial]: ../../demo/runs/oidc/2026-10-21-jellyfin-2/README.md

Field labels and restart instructions were checked against upstream's
[v4.0.0.4 configuration page](https://github.com/9p4/jellyfin-plugin-sso/blob/v4.0.0.4/SSO-Auth/Config/configPage.html),
and installation/Branding against its [README][plugin], on 2026-11-04.
