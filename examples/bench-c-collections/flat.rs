use prebindgen_proc_macro::{features, prebindgen, prebindgen_out_dir};
pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();
pub const FEATURES: &str = features!();

pub struct ResourceImpl(u64);
#[prebindgen]
pub type Resource = ResourceImpl;
#[prebindgen]
pub fn resource_value(value: &Resource) -> u64 { value.0 }
#[prebindgen]
pub struct Record { pub first: Resource, pub tag: u64, pub second: Resource }
#[prebindgen]
pub fn record_into(value: Record) -> Record { value }
#[prebindgen]
pub fn make_handles(n: usize) -> Vec<Resource> {
    std::hint::black_box((0..n).map(|i| ResourceImpl(i as u64)).collect())
}
#[prebindgen]
pub fn make_records(n: usize) -> Vec<Record> {
    std::hint::black_box((0..n).map(|i| Record {
        first: ResourceImpl(i as u64), tag: i as u64, second: ResourceImpl(i as u64 + 1)
    }).collect())
}
#[prebindgen]
pub fn make_nested(n: usize) -> Vec<Vec<Resource>> {
    std::hint::black_box(vec![vec![], make_handles(n), vec![]])
}
#[prebindgen]
pub fn make_many(n: usize) -> Vec<Vec<Resource>> {
    std::hint::black_box((0..n).map(|i| vec![ResourceImpl(i as u64)]).collect())
}
#[prebindgen]
pub fn make_numbers(n: usize) -> Vec<u64> {
    std::hint::black_box((0..n as u64).collect())
}
#[prebindgen]
pub fn control(n: usize) -> u64 { std::hint::black_box(n as u64) }

#[prebindgen]
pub fn resource_new(value:u64)->Resource {ResourceImpl(value)}
#[prebindgen]
pub fn record_new(first:Resource,tag:u64,second:Resource)->Record {Record{first,tag,second}}
#[prebindgen]
pub fn take_handles(values:Vec<Resource>)->u64 {values.iter().map(|v|v.0).sum()}
#[prebindgen]
pub fn take_records(values:Vec<Record>)->u64 {values.iter().map(|v|v.first.0+v.tag+v.second.0).sum()}
#[prebindgen]
pub struct WideInput {pub resource:Resource,pub a:u64,pub b:u64,pub c:u64,pub d:u64,pub e:u64,pub f:u64,pub g:u64}
#[prebindgen]
#[allow(clippy::too_many_arguments)]
pub fn wide_new(resource:Resource,a:u64,b:u64,c:u64,d:u64,e:u64,f:u64,g:u64)->WideInput {WideInput{resource,a,b,c,d,e,f,g}}
#[prebindgen]
pub fn take_wide(values:Vec<WideInput>)->u64 {values.iter().map(|v|v.resource.0+v.a+v.b+v.c+v.d+v.e+v.f+v.g).sum()}
#[prebindgen]
pub struct TextInput {pub resource:Resource,pub text:String}
#[prebindgen]
pub fn text_new(resource:Resource,text:String)->TextInput {TextInput{resource,text}}
#[prebindgen]
pub fn take_text(values:Vec<TextInput>)->u64 {values.iter().map(|v|v.resource.0+v.text.len() as u64).sum()}
#[prebindgen]
pub fn take_numbers(values:Vec<u64>)->u64 {values.iter().sum()}
