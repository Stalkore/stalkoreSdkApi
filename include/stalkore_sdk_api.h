#ifndef STALKORE_SDK_API_H
#define STALKORE_SDK_API_H
#include <stdint.h>
#if defined(_MSC_VER)
#define STALKORE_CALL __cdecl
#else
#define STALKORE_CALL
#endif
#ifdef __cplusplus
extern "C" {
#endif

/* All pointers are process-local. Never serialize tables, callbacks or handles. */
typedef int32_t StalkoreStatus;
enum {
    STALKORE_ABI_V1 = 1, STALKORE_CLIENT_SERVICE = 1, STALKORE_SCHEDULER_SERVICE = 2,
    STALKORE_PRESENTATION_SERVICE = 3,
    STALKORE_OK = 0, STALKORE_NOT_READY = 1, STALKORE_UNSUPPORTED_CLIENT = 2,
    STALKORE_INVALID_ARGUMENT = 3, STALKORE_WRONG_THREAD = 4, STALKORE_STALE_HANDLE = 5,
    STALKORE_INCOMPATIBLE_CLIENT = 6, STALKORE_INTERNAL_ERROR = 7,
    STALKORE_SDK_UNAVAILABLE = 8, STALKORE_UNSUPPORTED_VERSION = 9,
    STALKORE_UNKNOWN_SERVICE = 10, STALKORE_BUSY = 11,
    STALKORE_CHARACTER_EMPTY = 20, STALKORE_CHARACTER_DELETING = 21,
    STALKORE_CHARACTER_METHOD = 22, STALKORE_CHARACTER_DISPATCH = 23
};
#define STALKORE_CAP_LOGIN UINT64_C(1)
#define STALKORE_CAP_CHARACTER UINT64_C(2)
#define STALKORE_CAP_PIN UINT64_C(4)
enum {
    STALKORE_MODE_NONE = 0, STALKORE_MODE_LOGIN = 1, STALKORE_MODE_MAP = 2, STALKORE_MODE_OTHER = 3,
    STALKORE_PHASE_UNKNOWN = 0, STALKORE_PHASE_ACCOUNT_PROMPT = 1,
    STALKORE_PHASE_ACCOUNT_SELECTION = 2, STALKORE_PHASE_CREDENTIALS = 3,
    STALKORE_PHASE_AUTHENTICATING = 4, STALKORE_PHASE_SERVERS = 5,
    STALKORE_PHASE_CHARACTERS = 6, STALKORE_PHASE_MAP = 7,
    STALKORE_FLAG_LOGIN_READY = 1, STALKORE_FLAG_SERVER_COUNT = 2,
    STALKORE_FLAG_SSO_IDLE = 4, STALKORE_FLAG_MODAL = 8,
    STALKORE_FLAG_MAP_DISCONNECTED = 16, STALKORE_FLAG_PIN_PRESENT = 32,
    STALKORE_FLAG_PIN_READY = 64, STALKORE_FLAG_LOGIN_STAGE_VALID = 128,
    STALKORE_DIALOG_NONE = 0, STALKORE_DIALOG_DISCONNECTED = 1,
    STALKORE_DIALOG_CONNECTION_FAILURE = 2, STALKORE_PIN_CONFIRM = 10,
    STALKORE_EVENT_READY = 1, STALKORE_EVENT_TICK = 2,
    STALKORE_EVENT_STOPPED = 3, STALKORE_EVENT_FAILED = 4
};
typedef struct { uint32_t size, version; } StalkoreApiHeader;
typedef struct {
    StalkoreApiHeader header;
    uint64_t capabilities;
    uint32_t account_name_capacity, reserved;
} StalkoreClientInfoV1;
typedef struct {
    StalkoreApiHeader header;
    uint32_t mode, mode_token, phase, progress_revision, flags;
    int32_t server_count;
    uint32_t dialog_token, dialog_kind;
} StalkoreClientSnapshotV1;
/* Bytes are borrowed only until request_login returns; no trailing NUL. */
typedef struct { const uint8_t *data; uint32_t len; } StalkoreBytesV1;
typedef struct {
    StalkoreApiHeader header;
    StalkoreStatus (STALKORE_CALL *probe)(StalkoreClientInfoV1 *);
    StalkoreStatus (STALKORE_CALL *snapshot)(StalkoreClientSnapshotV1 *);
    StalkoreStatus (STALKORE_CALL *open_account_selection)(uint32_t mode);
    StalkoreStatus (STALKORE_CALL *select_account)(uint32_t mode);
    StalkoreStatus (STALKORE_CALL *request_login)(uint32_t mode, StalkoreBytesV1 user, StalkoreBytesV1 password);
    StalkoreStatus (STALKORE_CALL *select_server)(uint32_t mode, uint32_t index);
    StalkoreStatus (STALKORE_CALL *select_character)(uint32_t mode, uint32_t slot);
    StalkoreStatus (STALKORE_CALL *pin_ready)(uint32_t mode, uint32_t input_count);
    StalkoreStatus (STALKORE_CALL *pin_key)(uint32_t mode, uint32_t digit_or_confirm, uint32_t input_count);
    StalkoreStatus (STALKORE_CALL *acknowledge_dialog)(uint32_t dialog, uint32_t connection_failed);
    StalkoreStatus (STALKORE_CALL *return_to_login)(uint32_t mode);
} StalkoreClientApiV1;
typedef struct {
    StalkoreApiHeader header;
    uint32_t kind;
    StalkoreStatus status;
    uint32_t subscription, thread_id;
} StalkoreTickEventV1;
typedef StalkoreStatus (STALKORE_CALL *StalkoreTickCallbackV1)(void *context, const StalkoreTickEventV1 *event);
typedef struct {
    StalkoreApiHeader header;
    StalkoreStatus (STALKORE_CALL *subscribe)(StalkoreTickCallbackV1 callback, void *context, uint32_t interval_ms, uint32_t *subscription);
    StalkoreStatus (STALKORE_CALL *unsubscribe)(uint32_t subscription);
} StalkoreSchedulerApiV1;

enum { STALKORE_CAP_WINDOW_VISIBILITY = 1 };
typedef struct {
    StalkoreApiHeader header;
    uint32_t capabilities, visible;
} StalkorePresentationSnapshotV1;

/* Run on the SDK UI thread. Hiding alone implies no resource reduction. */
typedef struct {
    StalkoreApiHeader header;
    StalkoreStatus (STALKORE_CALL *snapshot)(StalkorePresentationSnapshotV1 *);
    StalkoreStatus (STALKORE_CALL *set_visible)(uint32_t visible);
} StalkorePresentationApiV1;
typedef StalkoreStatus (STALKORE_CALL *StalkoreQueryInterfaceV1)(uint32_t service, uint32_t version, const StalkoreApiHeader **output);
StalkoreStatus STALKORE_CALL StalkoreSdkQueryInterfaceV1(uint32_t service, uint32_t version, const StalkoreApiHeader **output);
#ifdef __cplusplus
}
#endif
#endif
