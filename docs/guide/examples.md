# Examples: see how they're made

Every example is a folder you can read in an evening. Use one, open one up,
run one, then make your own.

| Shape | What runs | Examples |
|---|---|---|
| **Carried** | A page in each person's browser. The group keeps its data. | Money, Events, House rules, Movie Night, Taste |
| **Lent** | A program on your computer. Your group opens it through svrngs. | Spot, Doc |
| **Yours alone** | A program on your computer, for you, that asks svrngs. | The pantry, the grapevine |

Each example's README has the same four parts: **Shape**, **Run it**, **How
it's made**, **Change it**.

## 1. Use one

On your home page, open your group, then **Add an app**. Tick **Money** and
press **Bring the ticked ones to** your group. More in [Apps](apps.md).

## 2. Open one up

Try changes in a group of your own. Whatever is in a group's folder reaches
everyone in it, and stays in its record.

1. Under **Make a group**, make one called `Sandbox`.
2. Bring Money to Sandbox.
3. Press **Open shared files**, then open `apps/Money/index.js`.
4. Change a sentence and save. Open Money again: it has changed.
5. Run `svrngs check ~/svrngs/Sandbox/apps/Money`. It says what svrngs would refuse.
6. Happy with it? Copy the folder into your real group's `apps/`.

**Folder location** on the group says where its folder really is.

## 3. Run one

Lent and yours-alone examples are programs. They need Node 24 or later and
svrngs's source at your version, nothing else.

1. Check Node: `node --version`. No Node, or older than 24? Get it from
   [nodejs.org](https://nodejs.org).
2. Get the examples that match your svrngs (1.11.0 or later; `svrngs update`
   first):
   ```
   v=$(svrngs version | cut -d' ' -f2)
   curl -L "https://github.com/alexsbryan/svrngs/archive/refs/tags/v$v.tar.gz" | tar xz
   cd "svrngs-$v"
   ```
3. Start the one you want. Its README's **Run it** has the line. Leave that
   terminal open: closing it stops the example.
4. A lent one: on your home page, under **Already running here**, press
   **Look for programs running on this computer**. Every program on your
   computer is listed; on the line naming your example's port (*Something
   at http://localhost:8090*), fill in **Its name:** and **Which
   program:**, tick your group, and press **Lend it to** your group. More
   in [Lend](lend.md).

## 4. Make your own

[Build an app](build.md) starts from two files: a carried guestbook, or a
lent server. `svrngs guide` prints that page in your terminal.
