#ifdef NDEBUG
#undef NDEBUG
#endif
#include <assert.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
static size_t numeric_loan_calls;
static void receive_numeric_loan(void *context, perftest_i64_slice_view values) {
    const perftest_numeric_loan_owner *owner=context;
    ++numeric_loan_calls;
    assert((uintptr_t)values.data==perftest_numeric_loan_address(owner, 0));
    if (values.len) assert(values.data[values.len-1]==(int64_t)values.len+6);
}
static void check_numeric_loans(void) {
    const size_t sizes[]={0,1,65536};
    for(size_t k=0;k<3;++k){
        size_t n=sizes[k];
        perftest_numeric_loan_owner *owner=perftest_numeric_loan_new(n);
        size_t capacity=perftest_numeric_loan_capacity(owner);
        perftest_i64_slice_view values=perftest_numeric_loan_values(owner);
        assert(values.len==n && (uintptr_t)values.data==perftest_numeric_loan_address(owner,0));
        if(n) assert(values.data[0]==7 && values.data[n-1]==(int64_t)n+6);
        perftest_f64_slice_view fixed=perftest_numeric_loan_fixed(owner);
        assert(fixed.len==3 && (uintptr_t)fixed.data==perftest_numeric_loan_address(owner,1));
        assert(fixed.data[0]==1.25 && signbit(fixed.data[1]) && isinf(fixed.data[2]));
        for(uint8_t kind=0;kind<3;++kind){
            perftest_i64_slice_view maybe={(const int64_t *)(uintptr_t)1,SIZE_MAX};
            int32_t present=perftest_numeric_loan_optional(owner,kind,&maybe);
            assert(present==(kind!=0));
            assert(maybe.len==(kind==2?n:0));
            if(kind) assert((uintptr_t)maybe.data==perftest_numeric_loan_address(owner,0));
            else assert(maybe.data==NULL);
        }
        char *error=NULL;
        perftest_i64_slice_view checked={0};
        assert(perftest_numeric_loan_checked(owner,0,&checked,&error));
        assert(!error && checked.data==values.data && checked.len==n);
        checked=(perftest_i64_slice_view){(const int64_t *)(uintptr_t)1,SIZE_MAX};
        assert(!perftest_numeric_loan_checked(owner,1,&checked,&error));
        assert((uintptr_t)checked.data==1 && checked.len==SIZE_MAX);
        assert(error && strstr(error,"no numeric loan")); perftest_free_error(error);
        numeric_loan_calls=0;
        perftest_numeric_loan_emit(owner,(perftest_numeric_loan_emit_callback){.run=receive_numeric_loan,.context=owner},3);
        assert(numeric_loan_calls==3);
        perftest_numeric_loan_copy_ret_w0_rows copied=perftest_numeric_loan_copy(owner);
        assert(copied.len==n);
        if(n){assert((void*)copied.data!=(void*)values.data);assert(copied.data[n-1].v0==(int64_t)n+6);}
        perftest_numeric_loan_copy_ret_w0_rows_free(copied);
        assert(perftest_numeric_loan_capacity(owner)==capacity);
        perftest_drop_numeric_loan_owner(owner);
        // Heap-backed views survive moving the inline Vec descriptor.
        perftest_owned_vec_i64_8f793c725f29723e vector=perftest_numeric_loan_vector(n);
        perftest_i64_slice_view view=perftest_owned_vec_i64_8f793c725f29723e_view(&vector);
        perftest_owned_vec_i64_8f793c725f29723e moved=vector;
        assert(perftest_owned_vec_i64_8f793c725f29723e_view(&moved).data==view.data);
        assert(view.len==n); if(n) assert(view.data[n-1]==(int64_t)n+6);
        perftest_drop_owned_vec_i64_8f793c725f29723e(&moved);
        perftest_i64_slice_view output={0};
        owner=perftest_numeric_loan_owned(n,&output);
        assert(output.len==n && (uintptr_t)output.data==perftest_numeric_loan_address(owner,0));
        perftest_drop_numeric_loan_owner(owner);
    }
    perftest_buffer_view bytes=perftest_numeric_loan_static_bytes();
    perftest_buffer_view text=perftest_numeric_loan_static_text();
    assert(bytes.len==14 && !memcmp(bytes.data,"original bytes",14));
    assert(text.len==13 && !memcmp(text.data,"original text",13));
    puts("C numeric loans PASS: original storage, arrays, presence, errors, callback loans, explicit copies and moved Vec owners");
}
