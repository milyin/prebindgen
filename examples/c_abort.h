#ifndef PREBINDGEN_C_ABORT_TEST_H
#define PREBINDGEN_C_ABORT_TEST_H
#ifndef _POSIX_C_SOURCE
#define _POSIX_C_SOURCE 200809L
#endif
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <signal.h>
#include <sys/resource.h>
#include <sys/wait.h>
#include <unistd.h>
/* Abort policies are tested in children so the smoke suite can continue. */
#define EXPECT_ABORT(expression) do { \
    FILE *log = tmpfile(); assert(log != NULL); \
    pid_t child = fork(); assert(child >= 0); \
    if (child == 0) { \
        struct rlimit limit = {0, 0}; setrlimit(RLIMIT_CORE, &limit); \
        assert(dup2(fileno(log), STDERR_FILENO) >= 0); \
        (void)(expression); _exit(0); \
    } \
    int status; assert(waitpid(child, &status, 0) == child); \
    assert(WIFSIGNALED(status) && WTERMSIG(status) == SIGABRT); \
    rewind(log); char line[1024]; int binding_failure = 0; \
    while (fgets(line, sizeof(line), log)) if (strstr(line, "C binding failed:") || strstr(line, "C destructor failed:") || strstr(line, "C error destructor failed:")) binding_failure = 1; \
    fclose(log); assert(binding_failure); \
} while (0)

#endif
