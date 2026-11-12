//! Selected physical fields in one native carrier; all ordering is computed
//! during generation. These predicates also preserve original optional presence
//! while an input carrier's pointer slots are emptied by ownership transfers.
use super::*;
use prebindgen_flat::WireNode;

#[derive(Clone)]
pub(crate) enum Test {
    Arm(syn::Ident, i32),
    Niche(syn::Ident, CNiche),
}
impl Test {
    fn name(&self) -> &syn::Ident {
        match self {
            Self::Arm(name, _) | Self::Niche(name, _) => name,
        }
    }
}
pub(crate) fn condition(
    tests: &[Test],
    mut value: impl FnMut(&syn::Ident) -> TokenStream,
) -> TokenStream {
    tests
        .iter()
        .map(|test| {
            let v = value(test.name());
            match test {
                Test::Arm(_, arm) => quote!((#v)==#arm),
                Test::Niche(_, niche) => niche.present(v),
            }
        })
        .reduce(|a, b| quote!((#a)&&(#b)))
        .unwrap_or_else(|| quote!(true))
}
pub(crate) fn values(
    node: &prebindgen_flat::WireNode<CWire, CChain, CBranch>,
) -> Result<Vec<(&Wire, Vec<Test>)>> {
    fn visit<'a>(
        node: &'a WireNode<CWire, CChain, CBranch>,
        path: &[Test],
        out: &mut Vec<(&'a Wire, Vec<Test>)>,
    ) {
        let mut add = |wires: &'a [Wire]| {
            for wire in wires {
                out.push((
                    wire,
                    path.iter()
                        .filter(|t| !matches!(t,Test::Niche(name,_) if *name==wire.name))
                        .cloned()
                        .collect(),
                ));
            }
        };
        match node {
            WireNode::Leaf(wires) => add(wires),
            WireNode::Chain {
                meta: CChain::Sequence(_) | CChain::Tagged(_),
                controls,
                ..
            } => add(controls),
            WireNode::Chain {
                meta: CChain::Optional(optional),
                child,
                controls,
            } => {
                add(controls);
                let mut next = path.to_vec();
                let name = optional.discriminator().clone();
                next.push(
                    optional
                        .niche()
                        .map(|n| Test::Niche(name.clone(), n.clone()))
                        .unwrap_or(Test::Arm(name, 1)),
                );
                visit(child, &next, out);
            }
            WireNode::Branch {
                meta,
                children,
                controls,
            } => {
                add(controls);
                for (i, child) in children.iter().enumerate() {
                    let mut next = path.to_vec();
                    if matches!(meta, CBranch::Enum(_) | CBranch::Alternatives) {
                        next.push(Test::Arm(controls[0].name.clone(), i as i32));
                    }
                    visit(child, &next, out);
                }
            }
        }
    }
    let mut values = vec![];
    visit(node, &[], &mut values);
    let indices = values
        .iter()
        .enumerate()
        .map(|(i, (w, _))| (w.name.to_string(), i))
        .collect::<std::collections::BTreeMap<_, _>>();
    fn add(
        i: usize,
        values: &[(&Wire, Vec<Test>)],
        indices: &std::collections::BTreeMap<String, usize>,
        state: &mut [u8],
        order: &mut Vec<usize>,
    ) -> Result<()> {
        if state[i] == 2 {
            return Ok(());
        }
        if state[i] == 1 {
            return Err("cyclic C field discriminator".into());
        }
        state[i] = 1;
        for test in &values[i].1 {
            let dependency = indices
                .get(&test.name().to_string())
                .ok_or("C discriminator outside its carrier")?;
            add(*dependency, values, indices, state, order)?;
        }
        state[i] = 2;
        order.push(i);
        Ok(())
    }
    let mut state = vec![0; values.len()];
    let mut order = vec![];
    for i in 0..values.len() {
        add(i, &values, &indices, &mut state, &mut order)?;
    }
    Ok(order.into_iter().map(|i| values[i].clone()).collect())
}
