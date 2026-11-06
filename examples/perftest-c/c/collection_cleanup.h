/* C-to-Rust callback results, using both checked and unchecked alias policies. */
struct collection_cleanup_state {int mode,calls,drops,releases;};
#define CC_PREFIX perftest
#define CC_CHECKED 1
#include "collection_cleanup_case.h"
#undef CC_CHECKED
#undef CC_PREFIX
#define CC_PREFIX fast
#define CC_CHECKED 0
#include "collection_cleanup_case.h"
#undef CC_CHECKED
#undef CC_PREFIX
static void check_collection_cleanup(void) {
    perftest_check_collection_cleanup();
    fast_check_collection_cleanup();
    puts("C collection callback cleanup PASS: nested rows, malformed descriptors, partial consumption, duplicate aliases and constructor failures");
}
