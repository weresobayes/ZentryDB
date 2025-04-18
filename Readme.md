# Zentry


a secure, fast, and minimal database engine for double-entry bookkeeping, purpose-built to serve as a transactional sentry in your system.


## 🧭 Why Zentry?

Zentry was born out of a recurring pattern I encountered across projects: the need for a reliable, purpose-built ledger to track transactions with double-entry guarantees — without dragging along the full weight of a general-purpose SQL database. While tools like PostgreSQL or MySQL are powerful, they often introduce unnecessary complexity and performance trade-offs when all you need is a lean, trustworthy, and auditable bookkeeping core.

Rather than forcing these domain rules into external schema constraints and ORM logic, Zentry embraces the domain at its core. It’s optimized from the ground up to do one thing well: **capture and preserve financial state with integrity**.

Zentry is designed to live alongside your services — as a sidecar, an embedded ledger, or a standalone transaction processor. Its binary storage format, B-Tree indexing, historical version tracking, and multi-system conversion model all serve the goal of making transactional state durable, understandable, and fast.

This is the ledger I always wanted to use — so I built it.

## 🗺️ Next milestones:

*Legends*

- ⬜️ = Not started
- ⬛️ = In progress
- ❌ = Cancelled / Can't be done
- ✅ = Completed

### ✅ Prefix Record Type on Key

To help distinguish record types during file scans and indexing, prefix the binary key with a tag:

- `C[USD -> IDR]` // C = Current ConversionGraph 
- `H[2025-04-12T07:16:23.479472314+00:00[IDR -> USD]2025-04-12T07:34:52.537529777+00:00]` // H = Historical ConversionGraph with effective range

This allows Zentry to quickly identify record categories when scanning the `.bin` file or rebuilding the index, and makes parsing more robust.

---

### ⬜ Compress Zeroing with Tombstone Byte

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