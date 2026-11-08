//! Only the measured fixture's pointer rows are supported; these are not production APIs.
type Address=(usize,bool);
pub fn sorted(values:&mut [Address])->Result<(),String>{
    values.sort_unstable_by_key(|e|e.0);
    if values.windows(2).any(|p|p[0].0==p[1].0&&(p[0].1||p[1].1)) {Err("conflicting input aliases".into())}else{Ok(())}
}
pub fn buckets(groups:&mut [Vec<Address>;5],cross:bool)->Result<(),String>{
    for group in groups.iter_mut(){sorted(group)?;}
    if !cross || groups.iter().filter(|g|!g.is_empty()).count()<=1{return Ok(());}
    // Fixed eight-leaf tournament: no merge buffer or heap allocation.
    let mut tree=[None::<(usize,usize)>;16];let mut pos=[0usize;5];
    for (i,g) in groups.iter().enumerate(){tree[8+i]=g.first().map(|e|(e.0,i));}
    fn least(a:Option<(usize,usize)>,b:Option<(usize,usize)>)->Option<(usize,usize)>{match(a,b){(None,b)=>b,(a,None)=>a,(Some(a),Some(b))=>Some(a.min(b))}}
    for i in (1..8).rev(){tree[i]=least(tree[i*2],tree[i*2+1]);}
    let mut previous=None::<Address>;
    while let Some((address,g))=tree[1]{
        let value=groups[g][pos[g]];
        if let Some(p)=previous{if p.0==address&&(p.1||value.1){return Err("conflicting input aliases".into());}}
        previous=Some(value);pos[g]+=1;let mut node=8+g;
        tree[node]=groups[g].get(pos[g]).map(|e|(e.0,g));
        while node>1{node/=2;tree[node]=least(tree[node*2],tree[node*2+1]);}
    }
    Ok(())
}
#[no_mangle]
pub extern "C" fn buckets_probe(kind:u32,address:usize)->bool{
    if kind==0 {sorted(&mut [(address,true),(address,false)]).is_ok()}
    else {let mut g=std::array::from_fn(|_|Vec::new());g[0].push((address,true));g[1].push((address,false));buckets(&mut g,kind==2).is_ok()}
}
