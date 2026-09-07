# How the tables relate

`FORMAT.md` describes the bytes. This document describes what the numbers mean
once decoded: how the 76 tables reference each other.

---

## Record index is not an ID

Two different numbers are easy to confuse.

**Record index** is the physical slot position in the file (0, 1, 2, ...). It is
an internal detail of one particular save: the same player sits at a different
slot in a different save. It is the `#` column in the grid and in exports, and
the `record_index` argument of `get_field` and `set_field`.

**Domain ID** (`playerid`, `teamid`, `leagueid`, ...) is a stable game
identifier, the same in every save and in every table that references it. This
is what links tables together — never the record index.

For example, Messi is always `playerid = 158023`, in `players`, in
`teamplayerlinks`, in `default_teamsheets`, in any save. His slot number in
`players` is arbitrary.

---

## ID glossary

| ID field | Owning table | Referenced by |
|---|---|---|
| `playerid` | `players` | `teamplayerlinks`, `editedplayernames`, `playersuspensions`, `playerloans`, `playerformdiff`, `player_grudgelove`, `playerpronouns`, `bannerplayers`, `default_teamsheets` (`playerid0`…`playerid50`, `captainid`, `penaltytakerid`, …) |
| `teamid` | `teams` | `teamplayerlinks`, `leagueteamlinks`, `teamnationlinks`, `teamstadiumlinks`, `teamformationteamstylelinks`, `manager`, `default_teamsheets`, `fixtures` (`hometeamid`/`awayteamid`), `teamcounterparts`, `teamsponsorlinks`, `rivals`, `competitionseeds`, `formations` |
| `leagueid` | `leagues` | `leagueteamlinks`, `teamnationlinks`, `referee`, `leaguerefereelinks`, `fixtures` |
| `competitionid` | `competition` | `fixtures`, `competitionseeds`, `competitionbadges`, `competitionkits`, `competitionmatchups`, `competitionstadiumlinks`, `competitionsponsorlinks`, `competitionballs` |
| `refereeid` | `referee` | `leaguerefereelinks`, `competitionrefereekits` |
| `formationid` | `formations` | `teamformationteamstylelinks` |
| `managerid` | `manager` | nothing — the link to a team runs the other way, via `manager.teamid` |

Two useful quirks:

- **National teams are ordinary `teams` rows.** There is no separate table for
  them, and their competitions are ordinary `leagues` rows.
- **`stadiumid` and `nationid` have no owning table in the save.** Stadium names
  are stored inline in `teamstadiumlinks`, so no join is needed. Nation names
  live in the game's main database, so a save gives you only the number.

---

## Common lookups

**A team's squad**: filter `teamplayerlinks` by `teamid`; each row gives a
`playerid`, a `jerseynumber` and a `position`. Then look up each `playerid` in
`players`.

**A team's league**: find the `leagueteamlinks` row with that `teamid` and read
its `leagueid`.

**A team's formation**: `formations` has a `teamid` column, so filter on it
directly. The 11 slots are `position0`…`position10` with `offset0x`/`offset0y`
through `offset10x`/`offset10y` as floating-point pitch coordinates.

**A team's starting eleven**: `default_teamsheets` holds `playerid0` onwards
plus role assignments such as `captainid` and `penaltytakerid`.

**Player names are not in the save.** `players` stores `firstnameid`,
`lastnameid` and `playerjerseynameid` — numeric references into the game's main
database. Load `fifa_ng_db.db` as a reference file to resolve them to text;
otherwise the editor can only show the numeric `playerid`.

---

## Tables without their own domain ID

Many tables are pure link tables: they exist only to join two IDs and carry no
identifier of their own. `leagueteamlinks`, `teamnationlinks`,
`teamstadiumlinks`, `competitionsponsorlinks` and similar all fall into this
group. Their primary key is effectively the pair of IDs they contain.
