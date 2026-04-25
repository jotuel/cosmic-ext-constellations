## 2024-05-30 - [Optimize bulk space filter]
**Learning:** To reduce RwLock contention, batch filtering logic utilizing iterators directly inside the lock method avoids overheads of massive `Vec` capacity preallocations and iterative atomic locks.
**Action:** When migrating N loops on locked traits, use internal iterator callbacks inside a scoped read guard instead of pre-collecting into N-size Vectors to fetch them piecemeal.
