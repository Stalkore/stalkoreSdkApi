#include <stddef.h>
#include "../include/stalkore_sdk_api.h"

_Static_assert(STALKORE_CLIENT_SERVICE == 1, "client service");
_Static_assert(STALKORE_SCHEDULER_SERVICE == 2, "scheduler service");
_Static_assert(STALKORE_PRESENTATION_SERVICE == 3, "presentation service");
_Static_assert(STALKORE_FRAME_PACING_SERVICE == 4, "frame pacing service");
_Static_assert(STALKORE_PROCESS_MEMORY_SERVICE == 5, "process memory service");
_Static_assert(sizeof(StalkoreClientInfoV1) == 24, "client info ABI");
_Static_assert(sizeof(StalkoreClientSnapshotV1) == 40, "client snapshot ABI");
_Static_assert(sizeof(StalkoreTickEventV1) == 24, "scheduler event ABI");
_Static_assert(sizeof(StalkorePresentationSnapshotV1) == 16, "presentation ABI");
_Static_assert(sizeof(StalkoreFramePacingSnapshotV1) == 40, "frame pacing ABI");
_Static_assert(sizeof(StalkoreMemoryCountersV1) == 24, "memory counters ABI");
_Static_assert(offsetof(StalkoreMemoryCountersV1, page_fault_count) == 16, "fault counter offset");
_Static_assert(sizeof(StalkoreProcessMemorySnapshotV1) == 48, "process memory ABI");
_Static_assert(offsetof(StalkoreProcessMemorySnapshotV1, context_token) == 16, "token offset");
_Static_assert(offsetof(StalkoreProcessMemorySnapshotV1, counters) == 24, "counters offset");
_Static_assert(sizeof(StalkoreResidencyReclaimResultV1) == 64, "reclaim result ABI");
_Static_assert(offsetof(StalkoreResidencyReclaimResultV1, elapsed_microseconds) == 8, "duration offset");
_Static_assert(offsetof(StalkoreResidencyReclaimResultV1, before) == 16, "before offset");
_Static_assert(offsetof(StalkoreResidencyReclaimResultV1, after) == 40, "after offset");
_Static_assert(offsetof(StalkoreProcessMemoryApiV1, snapshot) == 8, "snapshot method offset");
_Static_assert(sizeof(StalkoreProcessMemoryApiV1) == 8 + 2 * sizeof(void (*)(void)), "service table ABI");
