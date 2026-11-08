#ifdef NDEBUG
#undef NDEBUG
#endif
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "owners.h"
#define CHECK_SHARED_NUMERIC(type, suffix) do { \
    const type values[3] = {(type)1, (type)2, (type)3}; \
    assert(perftest_shared_input_##suffix((perftest_##suffix##_slice_view){values,3})==(uintptr_t)values); \
} while(0)
static void check_shared_inputs(void) {
    CHECK_SHARED_NUMERIC(int8_t,i8); CHECK_SHARED_NUMERIC(int16_t,i16);
    CHECK_SHARED_NUMERIC(int32_t,i32); CHECK_SHARED_NUMERIC(int64_t,i64);
    CHECK_SHARED_NUMERIC(intptr_t,isize); CHECK_SHARED_NUMERIC(uintptr_t,usize);
    CHECK_SHARED_NUMERIC(uint16_t,u16); CHECK_SHARED_NUMERIC(uint32_t,u32);
    CHECK_SHARED_NUMERIC(uint64_t,u64); CHECK_SHARED_NUMERIC(float,f32);
    CHECK_SHARED_NUMERIC(double,f64);
    const int64_t values[]={7,8,9};
    perftest_i64_slice_view input={values,3};
    uintptr_t address=0;char *error=NULL;
    assert(perftest_shared_input_checked(input,3,&address,&error));
    assert(!error && address==(uintptr_t)values);
    perftest_i64_slice_view echoed=perftest_shared_input_echo(input);
    assert(echoed.data==values && echoed.len==3);
    assert(perftest_shared_input_optional(0,(perftest_i64_slice_view){(const int64_t*)(uintptr_t)1,SIZE_MAX})==UINTPTR_MAX);
    assert(perftest_shared_input_optional(1,(perftest_i64_slice_view){NULL,0})==0);
    assert(perftest_shared_input_optional(1,input)==3);
    assert(perftest_shared_input_checked((perftest_i64_slice_view){values,0},0,&address,&error));
    assert(!error && address==(uintptr_t)values);
    assert(perftest_shared_input_checked((perftest_i64_slice_view){NULL,0},0,&address,&error));
    assert(!error && address!=0);
    assert(perftest_shared_input_checked((perftest_i64_slice_view){(const int64_t*)(uintptr_t)1,0},0,&address,&error));
    assert(!error && address!=0);
    const perftest_i64_slice_view invalid[]={
        {NULL,1},{(const int64_t*)((const uint8_t*)values+1),1},
        {values,SIZE_MAX},{(const int64_t*)(UINTPTR_MAX & ~(uintptr_t)7),2}
    };
    for(size_t i=0;i<sizeof(invalid)/sizeof(*invalid);i++){
        address=123;
        assert(!perftest_shared_input_checked(invalid[i],invalid[i].len,&address,&error));
        assert(address==123 && error);perftest_free_error(error);error=NULL;
    }
    const double fixed[]={1.25,2.5,3.75};double total=0;
    assert(perftest_shared_input_fixed((perftest_f64_slice_view){fixed,3},&total,&error));
    assert(total==7.5 && !error);total=123;
    assert(!perftest_shared_input_fixed((perftest_f64_slice_view){fixed,2},&total,&error));
    assert(total==123 && error);perftest_free_error(error);error=NULL;
    const uint8_t bytes[]={1,2,255};
    assert(perftest_shared_input_qualified_bytes((perftest_u8_slice_view){bytes,3})==(uintptr_t)bytes);
    assert(perftest_shared_input_text((perftest_buffer){bytes,0})!=0);
    const char text[]="shared text";
    assert(perftest_shared_input_text((perftest_buffer){(const uint8_t*)text,strlen(text)})==(uintptr_t)text);
    assert(perftest_shared_input_string((perftest_buffer){(const uint8_t*)text,strlen(text)})!=(uintptr_t)text);
    assert(perftest_shared_input_bytes((perftest_buffer){bytes,3})!=(uintptr_t)bytes);
    perftest_shared_input_copy_p0_w2_row copied_input[]={{7},{8},{9}};
    assert(perftest_shared_input_copy((perftest_shared_input_copy_p0_w2_rows){copied_input,3})!=(uintptr_t)copied_input);
    assert(perftest_shared_input_optional_vector(0,(perftest_shared_input_optional_vector_p0_w4_rows){NULL,SIZE_MAX})==UINTPTR_MAX);
    assert(perftest_shared_input_optional_vector(1,(perftest_shared_input_optional_vector_p0_w4_rows){NULL,0})==0);
    perftest_shared_input_rows_p0_w1_row rows[]={{input},{input}};
    assert(perftest_shared_input_rows((perftest_shared_input_rows_p0_w1_rows){rows,2})==20);
    // A view of Rust-owned storage can return to Rust while the owner remains
    // alive and shared. No move or overlapping exclusive access during the loan.
    perftest_numeric_loan_owner *owner=perftest_numeric_loan_new(3);
    input=perftest_numeric_loan_values(owner);echoed=perftest_shared_input_echo(input);
    assert(echoed.data==input.data && echoed.len==3);
    perftest_drop_numeric_loan_owner(owner);
    owner_numbers numbers=owner_inline_numbers_new();owner_i64_slice_view native=owner_numbers_view(&numbers);
    assert(owner_shared_input_vector(&numbers)==(uintptr_t)native.data);owner_drop_numbers(&numbers);
    owner_text string=owner_inline_text_new();owner_buffer_view string_view=owner_text_view(&string);
    assert(owner_shared_input_string(&string)==(uintptr_t)string_view.data);owner_drop_text(&string);
    owner_buffer_view byte_view={0};uintptr_t len=0;owner_bytes buffer=owner_inline_bytes_new(&byte_view,&len);
    assert(owner_shared_input_bytes(&buffer)==(uintptr_t)byte_view.data);owner_drop_bytes(&buffer);
    puts("C shared inputs PASS: original addresses, numeric widths, null/empty, malformed ranges, fixed arrays, optional inputs, constructors, native owners and explicit copies");
}
#undef CHECK_SHARED_NUMERIC
