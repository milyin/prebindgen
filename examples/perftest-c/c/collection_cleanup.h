/* C callback-result ownership cleanup, including source constructor failures. */
struct collection_cleanup_state {int mode,calls,drops,releases;};
#define CC_PREFIX perftest
#include "collection_cleanup_case.h"
#undef CC_PREFIX
static void check_collection_cleanup(void) {
    perftest_check_collection_cleanup();
    puts("C collection callback cleanup PASS: nested rows, optional owners, partial consumption and constructor failures");
}
