/* Real C callers exercise ownership through the generated public header. */
#define CLOSE_OUTPUT_ROWS(rows, free_rows) do { \
    for (size_t i=0; i<(rows).len; ++i) { \
        assert(perftest_callback_row_resource_id((rows).data[i].v0) >= 0); \
        perftest_drop_c_output_resource((rows).data[i].v0); \
        perftest_drop_owned_string(&(rows).data[i].v1); \
        perftest_drop_c_output_resource((rows).data[i].v2); \
    } \
    free_rows(rows); \
} while (0)

static void receive_output_rows(void *context, perftest_callback_4cf53ee6dc85aa46_callback_arg_0_w0_rows rows) {
    ++*(size_t *)context;
    for (size_t i=0; i<rows.len; ++i) {
        perftest_callback_4cf53ee6dc85aa46_callback_arg_0_w4_rows child=rows.data[i].v0;
        CLOSE_OUTPUT_ROWS(child,perftest_callback_4cf53ee6dc85aa46_callback_arg_0_w4_rows_free);
    }
    perftest_callback_4cf53ee6dc85aa46_callback_arg_0_w0_rows_free(rows);
}
static void check_output_owners(void) {
    assert(perftest_callback_row_resource_live()==0);
    for (int64_t fail=-1; fail<3; ++fail) {
        perftest_c_output_rows_ret_w2_rows rows={NULL,12345};
        char *error=NULL;
        bool ok=perftest_c_output_rows(fail,&rows,&error);
        assert(ok==(fail<0));
        if (ok) {
            assert(rows.len==3 && error==NULL);
            CLOSE_OUTPUT_ROWS(rows,perftest_c_output_rows_ret_w2_rows_free);
        } else {
            assert(rows.data==NULL && rows.len==12345 && error!=NULL);
            perftest_free_error(error);
        }
        assert(perftest_callback_row_resource_live()==0);
        size_t calls=0;
        perftest_c_output_callback(fail,(perftest_callback_4cf53ee6dc85aa46){receive_output_rows,&calls,NULL});
        assert(calls==(size_t)(fail<0));
        assert(perftest_callback_row_resource_live()==0);
    }
    puts("C output ownership PASS: partial rows, native String owners and buffered callback arguments");
}
#undef CLOSE_OUTPUT_ROWS
