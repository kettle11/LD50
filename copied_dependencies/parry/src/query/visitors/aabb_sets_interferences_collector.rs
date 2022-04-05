use crate::math::{Isometry, Matrix, Real};

/// Spatial partitioning data structure visitor collecting interferences with a given bounding volume.
pub struct AABBSetsInterferencesCollector<'a, T: 'a> {
    /// The transform from the local-space of the second bounding volumes to the local space of the first.
    pub ls_m2: &'a Isometry<Real>,
    /// The absolute value of the rotation matrix representing `ls_m2.rotation`.
    ///
    /// Equals to `ls_m2.rotation.to_rotation.matrix().matrix().abs()`.
    pub ls_m2_abs_rot: &'a Matrix<Real>,
    /// A tolerance applied to the interference tests.
    ///
    /// AABB pairs closer than `tolerance` will be reported as intersecting.
    pub tolerence: Real,
    /// The data contained by the nodes with bounding volumes intersecting `self.bv`.
    pub collector: &'a mut Vec<(T, T)>,
}

impl<'a, T> AABBSetsInterferencesCollector<'a, T> {
    /// Creates a new `AABBSetsInterferencesCollector`.
    #[inline]
    pub fn new(
        tolerence: Real,
        ls_m2: &'a Isometry<Real>,
        ls_m2_abs_rot: &'a Matrix<Real>,
        collector: &'a mut Vec<(T, T)>,
    ) -> AABBSetsInterferencesCollector<'a, T> {
        AABBSetsInterferencesCollector {
            tolerence,
            ls_m2,
            ls_m2_abs_rot,
            collector,
        }
    }
}

// impl<'a, T: Clone> SimultaneousVisitor<T, AABB> for AABBSetsInterferencesCollector<'a, T> {
//     #[inline]
//     fn visit(
//         &mut self,
//         left_bv: &AABB,
//         left_data: Option<&T>,
//         right_bv: &AABB,
//         right_data: Option<&T>,
//     ) -> VisitStatus {
//         let ls_right_bv = AABB::from_half_extents(
//             self.ls_m2 * right_bv.center(),
//             self.ls_m2_abs_rot * right_bv.half_extents() + Vector::repeat(self.tolerence),
//         );
//
//         if left_bv.intersects(&ls_right_bv) {
//             if let (Some(a), Some(b)) = (left_data, right_data) {
//                 self.collector.push((a.clone(), b.clone()))
//             }
//
//             VisitStatus::Continue
//         } else {
//             VisitStatus::Stop
//         }
//     }
// }
