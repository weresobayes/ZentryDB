## Next milestones:

### ✅ Prefix Record Type on Key

To help distinguish record types during file scans and indexing, prefix the binary key with a tag:

- `C[USD -> IDR]` // C = Current ConversionGraph 
- `H[2025-04-12T07:16:23.479472314+00:00[IDR -> USD]2025-04-12T07:34:52.537529777+00:00]` // H = Historical ConversionGraph with effective range

This allows Zentry to quickly identify record categories when scanning the `.bin` file or rebuilding the index, and makes parsing more robust.

---

### ✅ Compress Zeroing with Tombstone Byte

Rather than fully zeroing out old record space in the binary file, store a **1-byte tombstone prefix** at the beginning of each record:

- `0x01` → live record
- `0x00` → tombstoned (obsolete)

This allows Zentry to:
- Quickly skip dead records during index rebuilding
- Reduce write amplification compared to full zeroing
- Optionally reclaim space later with compaction

#### Binary Layout with Tombstone Byte

| Field              | Size     | Description                      |
|-------------------|----------|----------------------------------|
| Tombstone Flag     | 1 byte   | `0x01` = active, `0x00` = deleted |
| Key Length         | 2 bytes  | Length of key string             |
| Payload Length     | 4 bytes  | Length of the actual data        |
| Timestamp (optional)| 8 bytes | Created or rate_since            |
| Key + Payload      | N bytes  | Actual content                   |

This structure improves durability, simplifies forward compatibility, and sets the stage for future compaction or defragmentation strategies.

> 📝 This is optional for now. Keep it in the internal roadmap for Zentry v1.1 or later when introducing background compaction or snapshotting.

---