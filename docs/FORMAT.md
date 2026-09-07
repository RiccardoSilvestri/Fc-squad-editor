# The squad save format

This document describes the binary layout of an EA Sports FC 25 squad save, as
reconstructed by reading real files written by the game. Everything here is
verified by the test suite against a save the game accepts.

Squad files live in `%LOCALAPPDATA%\EA SPORTS FC 25\settings\` and are named
`Squads<timestamp>` with no extension.

---

## 1. Overall shape

A save is a small container header followed by an embedded relational database:

```
offset 0     FBCHUNKS container header
offset 218   container checksum (CRC-32)
offset 250   start of the checksummed payload
offset 254   "DB\0\x08" database signature
offset 278   table directory
             table 0
             table 1
             ...
             table 75
end of file
```

The database holds 76 tables: `players`, `teams`, `leagues`,
`teamplayerlinks`, `formations`, and so on.

---

## 2. Container header

The first 254 bytes are the container. Only three fields matter for editing:

| Offset | Size | Meaning |
|-------:|-----:|---------|
| 0 | 8 | Magic `FBCHUNKS` |
| 18 | variable | In-game save name, NUL-padded |
| 218 | 4 | Container checksum, little-endian u32 |

### The save name

The name shown in the game's load menu is plain ASCII at offset 18, terminated
and padded with NUL bytes. The space available is **not** fixed: it is the
length of the current name plus the run of NUL bytes that follows it. A save
named `funzionante` has 11 characters plus 2 NULs, so a new name can be at most
12 bytes. Writing past that overwrites live container data.

### The container checksum

The u32 at offset 218 is a **standard CRC-32** (reflected polynomial
`0xEDB88320`, initial value and final XOR `0xFFFFFFFF` — the same as zlib)
computed over **every byte from offset 250 to the end of the file**.

This is the single most important field for anyone writing an editor. The
database has its own internal checksums, and getting all of those right is not
enough: if this one is stale, the game reports the save as damaged and offers to
delete it. It must be recomputed after any change, as the very last step.

---

## 3. Table directory

The database signature `DB\0\x08` sits at offset 254. The directory starts 24
bytes later, at offset 278, and is a flat list of 8-byte entries:

```
[4 bytes] table short name, ASCII, e.g. "CZUM"
[4 bytes] offset of the table, little-endian u32, relative to the end of the directory
```

The list ends at the first entry whose name is not four alphanumeric characters
or whose offset goes backwards. The byte after the last entry is the base all
table offsets are relative to.

Table short names are opaque four-character codes. Their human-readable names
(`CZUM` = `players`, `lyxL` = `teams`, `RrqT` = `teamplayerlinks`) come from an
external `fifa_ng_db-meta.xml` schema file, not from the save itself. Without
that schema you can still read the data, but every field is identified only by a
four-character code.

---

## 4. Table layout

Tables are stored back to back with no padding: each table's `end` is exactly
the next table's `start`.

```
start +0     4 bytes   checksum of the PREVIOUS block (see section 6)
start +8     4 bytes   record size in bytes
start +12    4 bytes   record size in bits
start +16    4 bytes   compressed string length
start +20    2 bytes   number of record slots
start +22    2 bytes   number of records actually written
start +28    1 byte    number of fields
start +40              field descriptors, 16 bytes each
                       record data, `slots * record_size` bytes
                       optional trailing index block
end
```

### Live records

`slots` is the capacity; `written` is how many records are actually in use.
**The live records are always the first `written` slots, contiguous from index
0.** Slots past that point are not cleared by the game and still contain
leftovers from previous saves, which decode into meaningless values. Reading
them as if they were real data is the classic mistake: it invents hundreds of
phantom teams whose names are random bytes.

Note that `written` is a `u16`, so a table cannot hold more than 65535 records.
The largest table in a real save is `players` with 27000 slots.

### Field descriptors

Each field is 16 bytes:

```
+0   4 bytes   type
+4   4 bytes   bit offset within the record
+8   4 bytes   short name, ASCII
+12  4 bytes   width in bits
```

Types:

| Value | Type |
|------:|------|
| 0 | Fixed-length string |
| 3 | Integer |
| 4 | 32-bit float |
| 13 | Short compressed string |
| 14 | Long compressed string |

Descriptors are not stored in bit order, so sort by bit offset before laying
out a record.

---

## 5. Reading and writing values

**Integers** are bit-packed, LSB first, at an arbitrary bit offset and an
arbitrary width. A field 3 bits wide holds values 0 to 7 — writing 90 into it
would silently corrupt the neighbouring fields, so range checking matters. The
schema may also define a `rangelow` offset added to the stored value.

**Floats** are plain little-endian IEEE-754 at a byte-aligned offset.

**Fixed-length strings** occupy `width / 8` bytes, NUL-padded.

**Compressed strings** store a 4-byte offset in the record, relative to the end
of the record data. That points at a length header (1 byte for short strings, 2
big-endian bytes for long ones) followed by Huffman-coded bytes. The Huffman
tree lives at the end of the record data and is shared by the whole column.

**Dates** are days since 1582-10-14 12:00 UTC.

---

## 6. The checksum chain

Every table is covered by a 32-bit checksum, but not where you would expect.

The checksum for a table is stored at the table's `end` offset — which, because
tables are contiguous, is **the first four bytes of the next table's header**.
It covers the bytes from that table's field descriptors up to its end:

```
checksum_at(table[i].end) = crc(bytes from table[i].field_desc_start to table[i].end)
```

The chain starts before the first table: the four bytes at `table[0].start`
hold the checksum of the directory, covering `dir_start` to `table[0].start`.

The chain ends without a tail: the last table's `end` is the end of the file, so
its contents are not covered by any checksum.

A consequence worth stating plainly: the 40-byte table headers are themselves
outside every checksummed region. Corrupting a header is not detected by these
checksums at all.

### The algorithm

This is **not** a standard CRC-32. It is a non-reflected CRC using polynomial
`0x04C11DB7`, initial value `0xFFFFFFFF`, no final XOR, processing each byte
into the high end of the register:

```rust
fn checksum(bytes: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in bytes {
        crc ^= (byte as u32) << 24;
        for _ in 0..8 {
            crc = if crc & 0x8000_0000 != 0 {
                (crc << 1) ^ 0x04C1_1DB7
            } else {
                crc << 1
            };
        }
    }
    crc
}
```

Note this differs from the container checksum at offset 218, which is an
ordinary zlib CRC-32. A save file uses two different checksum algorithms.

---

## 7. Trailing index blocks

Sixteen of the 76 tables have extra bytes between the end of their record data
and the end of the table. This block appears to be a lookup index. Its exact
structure is not documented here because it has not been fully worked out.

The practical consequence: **appending records to those tables is not safe**,
because the index would have to be regenerated to match. `players` and
`teamplayerlinks` are both in this group. Editing existing records in them is
fine — only growing the table is a problem.

---

## 8. Writing a file the game will accept

In order:

1. Edit the record bytes.
2. Recompute the checksum of every table you touched, writing it at the table's
   `end` offset.
3. Recompute the container CRC-32 over offset 250 to end of file, and write it
   at offset 218.
4. Write the file.

Two further practical notes, learned the hard way:

- **Do not leave extra files in the game's save folder.** FC 25 tries to read
  every file there. A backup named `_1_Squads20260101120000000` is picked up,
  fails to parse as a save, and produces a "damaged file" prompt that looks like
  your edit broke something. Subdirectories are ignored, so put backups in one.
- **A save with correct table checksums but a stale container checksum is
  rejected**, and the error message says nothing about checksums. If the game
  loads the save name but then refuses the file, offset 218 is the first thing
  to check.
