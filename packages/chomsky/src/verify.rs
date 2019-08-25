use chomsky_extract::IKunTree;

pub struct IKunVerifier;

impl IKunVerifier {
    /// Verifies if the optimized tree preserves the intent of the original tree.
    /// For Phase 1, this is a basic structural check.
    pub fn verify(original: &IKunTree, optimized: &IKunTree) -> bool {
        // Basic intent preservation: if original is Map, optimized should also be some form of Map
        match (original, optimized) {
            (IKunTree::Map(f1, x1), IKunTree::Map(f2, x2)) => {
                Self::verify(f1, f2) && Self::verify(x1, x2)
            }
            (IKunTree::Map(f1, x1), IKunTree::GpuMap(f2, x2)) => {
                // GpuMap preserves Map intent
                Self::verify(f1, f2) && Self::verify(x1, x2)
            }
            (IKunTree::Reduce(f1, i1, l1), IKunTree::Reduce(f2, i2, l2)) => {
                Self::verify(f1, f2) && Self::verify(i1, i2) && Self::verify(l1, l2)
            }
            (IKunTree::Seq(s1), IKunTree::Seq(s2)) => {
                if s1.len() != s2.len() {
                    return false;
                }
                s1.iter().zip(s2.iter()).all(|(a, b)| Self::verify(a, b))
            }
            (IKunTree::Constant(v1), IKunTree::Constant(v2)) => v1 == v2,
            (IKunTree::StringConstant(s1), IKunTree::StringConstant(s2)) => s1 == s2,
            (IKunTree::Symbol(s1), IKunTree::Symbol(s2)) => s1 == s2,

            // Contexts can be added or removed during optimization if they are compatible
            (IKunTree::WithContext(_, b1), b2) => Self::verify(b1, b2),
            (b1, IKunTree::WithContext(_, b2)) => Self::verify(b1, b2),

            // Constraints should generally be preserved or strengthened
            (IKunTree::WithConstraint(_, b1), b2) => Self::verify(b1, b2),
            (b1, IKunTree::WithConstraint(_, b2)) => Self::verify(b1, b2),

            // If they are exactly the same, they are equivalent
            (a, b) if a == b => true,

            _ => false,
        }
    }
}
