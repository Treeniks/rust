use crate::errors;
use crate::mir::operand::OperandRef;
use crate::traits::*;
use rustc_middle::mir;
use rustc_middle::mir::interpret::{ConstValue, ErrorHandled};
use rustc_middle::ty::layout::HasTyCtxt;
use rustc_middle::ty::{self, Ty};
use rustc_span::source_map::Span;
use rustc_target::abi::Abi;

use super::FunctionCx;

impl<'a, 'tcx, Bx: BuilderMethods<'a, 'tcx>> FunctionCx<'a, 'tcx, Bx> {
    pub fn eval_mir_constant_to_operand(
        &self,
        bx: &mut Bx,
        constant: &mir::Constant<'tcx>,
    ) -> Result<OperandRef<'tcx, Bx::Value>, ErrorHandled> {
        let val = self.eval_mir_constant(constant)?;
        let ty = self.monomorphize(constant.ty());
        Ok(OperandRef::from_const(bx, val, ty))
    }

    pub fn eval_mir_constant(
        &self,
        constant: &mir::Constant<'tcx>,
    ) -> Result<ConstValue<'tcx>, ErrorHandled> {
        let ct = self.monomorphize(constant.literal);
        let uv = match ct {
            mir::ConstantKind::Ty(ct) => match ct.kind() {
                ty::ConstKind::Unevaluated(uv) => uv.expand(),
                ty::ConstKind::Value(val) => {
                    return Ok(self.cx.tcx().valtree_to_const_val((ct.ty(), val)));
                }
                err => span_bug!(
                    constant.span,
                    "encountered bad ConstKind after monomorphizing: {:?}",
                    err
                ),
            },
            mir::ConstantKind::Unevaluated(uv, _) => uv,
            mir::ConstantKind::Val(val, _) => return Ok(val),
        };

        self.cx.tcx().const_eval_resolve(ty::ParamEnv::reveal_all(), uv, None).map_err(|err| {
            match err {
                ErrorHandled::Reported(_) => {
                    self.cx.tcx().sess.emit_err(errors::ErroneousConstant { span: constant.span });
                }
                ErrorHandled::TooGeneric => {
                    self.cx
                        .tcx()
                        .sess
                        .diagnostic()
                        .emit_bug(errors::PolymorphicConstantTooGeneric { span: constant.span });
                }
            }
            err
        })
    }

    /// process constant containing SIMD shuffle indices
    pub fn simd_shuffle_indices(
        &mut self,
        bx: &Bx,
        span: Span,
        ty: Ty<'tcx>,
        constant: Result<Option<ty::ValTree<'tcx>>, ErrorHandled>,
    ) -> (Bx::Value, Ty<'tcx>) {
        let val = constant
            .ok()
            .flatten()
            .map(|val| {
                let field_ty = ty.builtin_index().unwrap();
                let values: Vec<_> = val
                    .unwrap_branch()
                    .iter()
                    .map(|field| {
                        if let Some(prim) = field.try_to_scalar() {
                            let layout = bx.layout_of(field_ty);
                            let Abi::Scalar(scalar) = layout.abi else {
                                bug!("from_const: invalid ByVal layout: {:#?}", layout);
                            };
                            bx.scalar_to_backend(prim, scalar, bx.immediate_backend_type(layout))
                        } else {
                            bug!("simd shuffle field {:?}", field)
                        }
                    })
                    .collect();
                bx.const_struct(&values, false)
            })
            .unwrap_or_else(|| {
                bx.tcx().sess.emit_err(errors::ShuffleIndicesEvaluation { span });
                // We've errored, so we don't have to produce working code.
                let ty = self.monomorphize(ty);
                let llty = bx.backend_type(bx.layout_of(ty));
                bx.const_undef(llty)
            });
        (val, ty)
    }
}
