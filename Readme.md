# Zentry


A secure, fast, and minimal database engine for double-entry bookkeeping, purpose-built to serve as a transactional sentry in your system.


## 🧭 Why Zentry?

Zentry was born out of a recurring pattern I encountered across projects: the need for a reliable, purpose-built ledger to track transactions with double-entry guarantees — without dragging along the full weight of a general-purpose SQL database. While tools like PostgreSQL or MySQL are powerful, they often introduce unnecessary complexity and performance trade-offs when all you need is a lean, trustworthy, and auditable bookkeeping core.

Rather than forcing these domain rules into external schema constraints and ORM logic, Zentry embraces the domain at its core. It’s optimized from the ground up to do one thing well: **capture and preserve financial state with integrity**.

Zentry is designed to live alongside your services — as a sidecar, an embedded ledger, or a standalone transaction processor. Its binary storage format, B-Tree indexing, historical version tracking, and multi-system conversion model all serve the goal of making transactional state durable, understandable, and fast.

This is the ledger I always wanted to use — so I built it.

## 🗺️ Technical Implementation Milestones

**Status Legend**: ⬜️ Not started | ⬛️ In progress | ❌ Cancelled | ✅ Completed

---

### ✅ Record Type Prefixing

**Spec**: Binary key prefixing for type identification

**Format**:
- `C[key]` - Current records (e.g., `C[USD -> IDR]`)
- `H[timestamp1[key]timestamp2]` - Historical records with effective range

**Benefits**:
- O(1) record type detection during scanning
- Simplified index rebuilding process
- Type-based filtering without payload inspection

---

### Binary Record Layout

#### ⬛️ Tombstone Implementation

**Spec**: 1-byte record state indicator
- `0x01`: Active record
- `0x00`: Tombstoned (deleted) record

**Benefits**:
- Efficient dead record skipping
- Reduced write amplification
- Enables space reclamation via compaction

#### ⬜ Complete Binary Layout

**Status**: Partially implemented, verification required

| Field            | Size     | Description                    |
|-----------------|----------|--------------------------------|
| Tombstone       | 1 byte   | `0x01`=active, `0x00`=deleted  |
| Key Length      | 2 bytes  | Length of prefixed key         |
| Payload Length  | 4 bytes  | Data length                    |
| Timestamp       | 8 bytes  | Optional: created/effective    |
| Key + Payload   | N bytes  | Content                        |

**Roadmap**: 
- v1.0: Core binary layout with tombstone support
- v1.1+: Compaction and snapshotting optimizations
