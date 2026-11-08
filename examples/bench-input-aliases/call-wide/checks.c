#define _GNU_SOURCE
#include "bindings.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

static bench_history_callback_callback_return malformed(void *context) {
    return *(bench_history_callback_callback_return *)context;
}
static void released(void *context,const bench_history_callback_callback_return *value) {
    (void)context;(void)value;fputs("RELEASED\n",stderr);
}
int main(int argc,char **argv) {
    assert(argc==2);
    if(!strcmp(argv[1],"shared")) {
        bench_resource *handle=bench_resource_new(7),*token=bench_resource_new(9);
        bench_history_readonly_p0_w5_row leaf[]={{handle},{handle}};
        bench_history_readonly_p0_w1_row row[]={{{leaf,2}}};
        assert(bench_history_readonly((bench_history_readonly_p0_w1_rows){row,1})==14);
        bench_history_shared_p0_w5_row shared[]={{handle},{handle}};
        bench_history_shared_p0_w1_row parent[]={{{shared,2}}};
        assert(bench_history_shared((bench_history_shared_p0_w1_rows){parent,1},token)==23);
        bench_drop_resource(token);bench_drop_resource(handle);return 0;
    }
    if(!strcmp(argv[1],"null")) {
        bench_history_flat_p0_w1_row row[]={{NULL}};
        bench_history_flat((bench_history_flat_p0_w1_rows){row,1});
    } else if(!strcmp(argv[1],"range")) {
        bench_history_flat((bench_history_flat_p0_w1_rows){NULL,1});
    } else if(!strcmp(argv[1],"overflow")) {
        bench_history_flat_p0_w1_row row[]={{NULL}};
        bench_history_flat((bench_history_flat_p0_w1_rows){row,SIZE_MAX});
    } else if(!strcmp(argv[1],"callback-range")) {
        bench_history_callback_callback_return rows={NULL,1};
        bench_history_callback((bench_history_callback_callback){.run=malformed,.context=&rows,.release_result=released});
    } else {assert(!"unknown case");}
    return 90;
}
