//! Active physical values in one native row. The tree supplies all predicates;
//! no runtime ownership registry or row-by-row plan is required.
use super::*;
use crate::kotlin::JniNode;
use prebindgen_flat::WireNode;

#[derive(Clone, Debug)]
pub(crate) enum Test {
    Arm(usize, i32),
    Present(usize, Option<JniNiche>),
}
impl Test {
    pub(crate) fn index(&self) -> usize {
        match self {
            Self::Arm(i, _) | Self::Present(i, _) => *i,
        }
    }
    pub(crate) fn rust(&self, wire: &JniWire, value: TokenStream) -> TokenStream {
        match self {
            Self::Arm(_, arm) => quote!((#value)==#arm),
            Self::Present(_, Some(niche)) => niche.present(wire, value),
            Self::Present(_, None) => quote!((#value)==1),
        }
    }
    pub(crate) fn kotlin(&self, value: &str) -> String {
        match self {
            Self::Arm(_, arm) => format!("({value}) == {arm}"),
            Self::Present(_, Some(JniNiche::Null)) => format!("({value}) != null"),
            Self::Present(_, Some(JniNiche::Integer(n))) => format!(
                "({value}).toLong() != {}",
                crate::wire::jvm::long_literal(*n)
            ),
            Self::Present(_, None) => format!("({value}) == 1"),
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Selection {
    pub(crate) tests: Vec<Vec<Test>>,
    kinds: Vec<JniWire>,
}
impl Selection {
    pub(crate) fn new(node: &JniNode) -> Result<Self> {
        let wires = group::row_fields(node).collect::<Vec<_>>();
        let indices = wires
            .iter()
            .enumerate()
            .map(|(i, w)| (w.name.to_string(), i))
            .collect::<BTreeMap<_, _>>();
        let mut tests = vec![vec![]; wires.len()];
        fn visit(
            node: &JniNode,
            path: &[Test],
            indices: &BTreeMap<String, usize>,
            out: &mut [Vec<Test>],
        ) -> Result<()> {
            let set = |wires: &[prebindgen_flat::Wire<JniWire>], out: &mut [Vec<Test>]| {
                for w in wires {
                    out[indices[&w.name.to_string()]] = path.to_vec();
                }
            };
            match node {
                WireNode::Leaf(wires) => set(wires, out),
                WireNode::Chain {
                    controls,
                    meta: JniChain::Sequence(_) | JniChain::Bulk(_),
                    ..
                } => set(controls, out),
                WireNode::Chain {
                    controls,
                    child,
                    meta: JniChain::Optional(optional),
                } => {
                    set(controls, out);
                    let mut next = path.to_vec();
                    let donor = indices[&optional.discriminator().to_string()];
                    next.push(Test::Present(donor, optional.niche().cloned()));
                    visit(child, &next, indices, out)?;
                    // The discriminator must be readable before testing presence.
                    out[donor].retain(|t| !matches!(t,Test::Present(i,_) if *i==donor));
                }
                WireNode::Branch {
                    controls,
                    children,
                    meta,
                } => {
                    set(controls, out);
                    for (i, child) in children.iter().enumerate() {
                        let mut next = path.to_vec();
                        if matches!(meta, JniBranch::Enum(_) | JniBranch::Alternatives) {
                            next.push(Test::Arm(indices[&controls[0].name.to_string()], i as i32));
                        }
                        visit(child, &next, indices, out)?;
                    }
                }
            }
            Ok(())
        }
        visit(node, &[], &indices, &mut tests)?;
        Ok(Self {
            tests,
            kinds: wires.iter().map(|w| w.kind.clone()).collect(),
        })
    }
    pub(crate) fn rust(
        &self,
        index: usize,
        mut value: impl FnMut(usize) -> TokenStream,
    ) -> TokenStream {
        self.tests[index]
            .iter()
            .map(|t| t.rust(&self.kinds[t.index()], value(t.index())))
            .reduce(|a, b| quote!((#a)&&(#b)))
            .unwrap_or_else(|| quote!(true))
    }
    pub(crate) fn kotlin(&self, index: usize, mut value: impl FnMut(usize) -> String) -> String {
        let tests = self.tests[index]
            .iter()
            .map(|t| t.kotlin(&value(t.index())))
            .collect::<Vec<_>>();
        if tests.is_empty() {
            "true".into()
        } else {
            tests.join(" && ")
        }
    }
    pub(crate) fn order(&self) -> Result<Vec<usize>> {
        fn add(i: usize, s: &Selection, states: &mut [u8], out: &mut Vec<usize>) -> Result<()> {
            if states[i] == 2 {
                return Ok(());
            }
            if states[i] == 1 {
                return Err("cyclic selected-value discriminator".into());
            }
            states[i] = 1;
            for t in &s.tests[i] {
                add(t.index(), s, states, out)?;
            }
            states[i] = 2;
            out.push(i);
            Ok(())
        }
        let mut states = vec![0; self.tests.len()];
        let mut order = vec![];
        for i in 0..states.len() {
            add(i, self, &mut states, &mut order)?;
        }
        Ok(order)
    }
}
