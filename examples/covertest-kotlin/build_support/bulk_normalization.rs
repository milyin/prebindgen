//! Generator-author coverage for redundant bulk normalization.
use prebindgen_jni::prelude::*;
use prebindgen_jni::{JniInputBranch, JniInputConfig, JniInputLeafOp, JniInputUnary};

// Deliberate raw-tree coverage: redundant bulk wrappers must normalize away.
pub fn redundant_bulk_replay_input_with(
    redundant_bulk: bool,
    bulk_rows: bool,
) -> prebindgen_jni::JniInputConfig {
    let borrowed = |function: &str| JniInputConfig::Unary {
        operation: JniInputUnary::Optional,
        child: Box::new(JniInputConfig::Branch {
            operation: JniInputBranch::Constructor {
                function: function.into(),
            },
            children: vec![JniInputConfig::Leaf(JniInputLeafOp::Value)],
        }),
    };
    let buffer = |function: &str| {
        let input = borrowed(function);
        if bulk_rows {
            input
        } else {
            input.bulk()
        }
    };
    let input = JniInputConfig::Unary {
        operation: JniInputUnary::Sequence,
        child: Box::new(JniInputConfig::Branch {
            operation: JniInputBranch::Record,
            children: vec![
                JniInputConfig::Leaf(JniInputLeafOp::Value),
                if redundant_bulk || !bulk_rows {
                    borrowed("bulk_replay_text").bulk()
                } else {
                    borrowed("bulk_replay_text")
                },
                buffer("bulk_replay_bytes"),
                borrowed("bulk_replay_kind"),
                JniInputConfig::Leaf(JniInputLeafOp::Value),
            ],
        }),
    };
    if bulk_rows {
        input.bulk()
    } else {
        input
    }
}
