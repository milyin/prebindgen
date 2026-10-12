#include "demo.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

static demo_buffer text(const char *s) { return (demo_buffer){(const uint8_t *)s,strlen(s)}; }
static const demo_buffer none={NULL,0};
static void equals(demo_buffer actual,const char *expected) {
 assert(actual.len==strlen(expected));assert(memcmp(actual.data,expected,actual.len)==0);demo_free_buffer(actual);
}
static uint64_t key(const char *s) {demo_keyexpr_from_string_result r;assert(demo_keyexpr_from_string(text(s),&r)==0);return r.v0;}
static uint64_t payload(const char *k,const char *d) {demo_payload_from_string_result r;assert(demo_payload_from_string(1,0,text(k),text(d),&r)==0);return r.v0;}
static uint64_t bytes(const char *s) {demo_bytes_from_slice_result r;assert(demo_bytes_from_slice(text(s),&r)==0);return r.v0;}

int main(void) {
 int cases=0;
 for(int kt=0;kt<2;kt++) for(int xt=0;xt<4;xt++) for(int xkt=0;xkt<(xt==0?1:2);xkt++) {
  uint64_t k=kt==0?key("k"):0;
  uint64_t x=xt==0?payload("x","A"):0;
  uint64_t xk=xt!=0&&xkt==0?key("x"):0;
  uint64_t d=xt==1?bytes("A"):0;
  uint64_t y=payload("y","B");
  demo_foo_result r;
  assert(demo_foo(kt,k,kt==1?text("k"):none,
    xt,x,
    xkt,xt==1?xk:0,xt==1&&xkt==1?text("x"):none,d,
    xkt,xt==2?xk:0,xt==2&&xkt==1?text("x"):none,xt==2?text("A"):none,
    xkt,xt==3?xk:0,xt==3&&xkt==1?text("x"):none,xt==3?text("A"):none,
    y,&r)==0);
  equals(r.v2,"k:x:y");
  // Child handles remain valid after consuming the original payload.
  demo_payload_text_result whole;assert(demo_payload_text(r.v0,&whole)==0);equals(whole.v0,"k:x:y:AB");
  demo_keyexpr_text_result part;assert(demo_keyexpr_text(r.v1,&part)==0);equals(part.v0,"k:x:y");
  demo_bytes_read_result data;assert(demo_bytes_read(r.v3,&data)==0);equals(data.v0,"AB");
  assert(demo_payload_text(y,&whole)==-1); // y was consumed.
  cases++;
 }
 assert(cases==14);
 // Only the selected constructor executes: invalid inactive pointers are inert.
 demo_payload_from_string_result inactive;
 demo_buffer invalid={(const uint8_t *)1,10};
 assert(demo_payload_from_string(0,key("safe"),invalid,text("value"),&inactive)==0);
 assert(demo_drop_payload(inactive.v0)==0);
 // Reject invalid tags before touching their children; caller retains these handles.
 uint64_t retained=key("retained");
 assert(demo_payload_from_string(99,retained,invalid,text("x"),&inactive)==-1);
 assert(strstr(demo_last_error(),"selector")!=NULL);
 assert(demo_drop_key(retained)==0);
 // Wrong type does not consume a handle; a stale handle never dereferences memory.
 uint64_t data_handle=bytes("x");demo_keyexpr_text_result wrong;
 assert(demo_keyexpr_text(data_handle,&wrong)==-1);
 assert(demo_drop_bytes(data_handle)==0);
 assert(demo_keyexpr_text(data_handle,&wrong)==-1);
 assert(demo_drop_bytes(data_handle)==0); // close is idempotent.
 // Invalid UTF-8 and null buffers fail through the error channel.
 uint8_t bad=255;demo_keyexpr_from_string_result kr;
 assert(demo_keyexpr_from_string((demo_buffer){&bad,1},&kr)==-1);
 assert(demo_keyexpr_from_string((demo_buffer){NULL,1},&kr)==-1);
 assert(demo_keyexpr_from_string(text("ok"),NULL)==-1);
 // Earlier consumed inputs stay consumed after a later failure.
 uint64_t early=key("early");demo_foo_result r;
 assert(demo_foo(0,early,none,99,0,0,0,none,0,0,0,none,none,0,0,none,none,0,&r)==-1);
 assert(demo_keyexpr_text(early,&wrong)==-1);
 // Reusing one owning payload twice fails before the source operation.
 uint64_t aliased=payload("a","A");
 assert(demo_foo(1,0,text("k"),0,aliased,0,0,none,0,0,0,none,none,0,0,none,none,aliased,&r)==-1);
 // Source panics are contained, and all consumed values are dropped.
 assert(demo_foo(1,0,text("panic"),0,payload("x","A"),0,0,none,0,0,0,none,none,0,0,none,none,payload("y","B"),&r)==-1);
 assert(strstr(demo_last_error(),"panicked")!=NULL);
 demo_noop_result unit;assert(demo_noop(&unit)==0);assert(strcmp(demo_last_error(),"")==0);
 puts("C PASS: 14 combinations, output ownership, inactive branches and failure paths");
}
